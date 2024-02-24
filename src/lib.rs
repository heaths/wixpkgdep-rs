// Copyright 2023 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use std::{
    collections::HashSet,
    fmt::Display,
    ops::{BitAnd, BitOr},
    str::FromStr,
};

use windows::{
    core::{w, PCWSTR},
    Win32::System::Registry::HKEY,
};

mod error;
mod provider;
mod registry;
mod version;

pub use error::Error;
pub use provider::{Dependency, Provider};
pub use version::Version;

use registry::map_registry_error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Scope {
    User,

    #[default]
    Machine,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[repr(u32)]
pub enum Attributes {
    #[default]
    None,

    MinVersionInclusive = 0x100,
    MaxVersionInclusive = 0x200,
}

const ROOT_KEY: PCWSTR = w!("Software\\Classes\\Installer\\Dependencies");
const DEPENDENTS_KEY: PCWSTR = w!("Dependents");

/// Gets information about a provider.
pub fn get_provider<K>(provider_key: K, scope: Scope) -> Result<Provider>
where
    K: AsRef<str> + Into<String>,
{
    let key =
        registry::Key::open::<HKEY, PCWSTR>(scope.into(), ROOT_KEY).map_err(map_registry_error)?;

    let _provider_key = to_pcwstr(provider_key.as_ref());
    let key = key
        .open_subkey::<PCWSTR>(_provider_key)
        .map_err(map_registry_error)?;

    Provider::from(provider_key, &key)
}

/// Checks that the dependency is registered and within the requested version range.
pub fn check_dependencies<K>(
    provider_key: K,
    scope: Scope,
    min_version: Option<Version>,
    max_version: Option<Version>,
    attributes: Option<Attributes>,
    dependencies: &mut HashSet<Dependency>,
) -> Result<()>
where
    K: AsRef<str> + Into<String>,
{
    // Equivalent to deputil:DepCheckDependency.
    let key = registry::Key::open::<HKEY, PCWSTR>(scope.into(), ROOT_KEY)
        .map_err(Error::RegistryError)?;

    // If the key or its Version value is missing, add it to the set of dependencies, and return NotFound.
    let _provider_key = to_pcwstr(provider_key.as_ref());
    let version: Version;
    match key
        .open_subkey::<PCWSTR>(_provider_key)
        .map_err(map_registry_error)
    {
        Ok(k) => {
            if let Ok(_version) = key.value(w!("Version")).map(|v| v.to_version())? {
                version = _version;
            } else {
                // We only have the provider key at this time.
                dependencies.insert(Dependency::new(provider_key));
                return Err(Error::NotFound);
            }
            k
        }
        Err(Error::NotFound) => {
            // We only have the provider key at this time.
            dependencies.insert(Dependency::new(provider_key));
            return Err(Error::NotFound);
        }
        Err(err) => return Err(err),
    };

    // Since the provider and Version were found, check the version range requirements.
    let dependency = Dependency::new(provider_key);
    if let Some(min_version) = min_version {
        let allow_equal = (attributes.unwrap_or_default() & Attributes::MinVersionInclusive)
            == Attributes::MinVersionInclusive as u32;

        if !(allow_equal && min_version <= version || min_version < version) {
            dependencies.insert(dependency);
            return Err(Error::NotFound);
        }
    }

    if let Some(max_version) = max_version {
        let allow_equal = (attributes.unwrap_or_default() & Attributes::MaxVersionInclusive)
            == Attributes::MaxVersionInclusive as u32;

        if !(allow_equal && version <= max_version || version < max_version) {
            dependencies.insert(dependency);
            return Err(Error::NotFound);
        }
    }

    Ok(())
}

/// Checks that there are no dependents registered for providers that are being uninstalled.
pub fn check_dependents<K>(
    provider_key: K,
    scope: Scope,
    #[allow(unused_variables)] // Prevent future breaking change; not currently used.
    attributes: Option<Attributes>,
    ignore: Option<&HashSet<String>>,
) -> Result<Option<Vec<Dependency>>>
where
    K: AsRef<str>,
{
    // Equivalent to deputil:DepCheckDependents.

    // Failure to open a provider or its Dependents key means no dependents.
    let key = match registry::Key::open::<HKEY, PCWSTR>(scope.into(), ROOT_KEY)
        .map_err(map_registry_error)
    {
        Err(Error::NotFound) => return Ok(None),
        err => err,
    }?;

    let provider_key = to_pcwstr(provider_key);
    let key = match key
        .open_subkey::<PCWSTR>(provider_key)
        .map_err(map_registry_error)
    {
        Err(Error::NotFound) => return Ok(None),
        err => err,
    }?;

    let key = match key
        .open_subkey::<PCWSTR>(DEPENDENTS_KEY)
        .map_err(map_registry_error)
    {
        Err(Error::NotFound) => return Ok(None),
        err => err,
    }?;

    Ok(Some(
        key.keys()?
            .filter_map(|k| {
                if let Some(ignore) = ignore {
                    if ignore.contains(&k.name) {
                        return None;
                    }
                }

                // BUGBUG: Should we check that the provider actually exists in case it didn't clean up during uninstall or was that meant for permanent packages?
                Some(Dependency::new(&k.name))
            })
            .collect(),
    ))
}

impl BitAnd for Attributes {
    type Output = u32;
    // cspell:ignore bitand
    fn bitand(self, rhs: Self) -> Self::Output {
        (self as u32) & (rhs as u32)
    }
}

impl BitOr for Attributes {
    type Output = u32;
    fn bitor(self, rhs: Self) -> Self::Output {
        (self as u32) | (rhs as u32)
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::User => write!(f, "user"),
            Scope::Machine => write!(f, "machine"),
        }
    }
}

impl FromStr for Scope {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" => Ok(Scope::User),
            "machine" => Ok(Scope::Machine),
            _ => Err(Error::NotSupported),
        }
    }
}

impl From<Scope> for windows::Win32::System::Registry::HKEY {
    fn from(value: Scope) -> Self {
        match value {
            Scope::User => registry::HKEY_CURRENT_USER,
            Scope::Machine => registry::HKEY_LOCAL_MACHINE,
        }
    }
}

fn to_pcwstr(value: impl AsRef<str>) -> PCWSTR {
    let value: Vec<u16> = value
        .as_ref()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    PCWSTR::from_raw(value.as_ptr())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pcwstr() {
        let value = to_pcwstr("test");
        let (_, value, _) = unsafe { value.as_wide().align_to::<u8>() };
        assert_eq!(value, b"t\0e\0s\0t\0");
    }
}
