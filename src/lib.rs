// Copyright 2023 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use std::{fmt::Display, str::FromStr};

use windows::{
    core::{w, PCWSTR},
    Win32::System::Registry::HKEY,
};

mod registry;
mod version;

pub use version::Version;

#[derive(Debug, Default, Clone)]
pub struct Provider {
    /// Provider key that uniquely identifies the provider.
    key: String,

    /// Optional identifier of the package for an external system e.g., a ProductCode for a Windows Installer package.
    #[allow(dead_code)] // TODO
    id: Option<String>,

    /// Optional display name of the provider.
    name: Option<String>,

    /// Optional version of the provider.
    #[allow(dead_code)] // TODO
    version: Option<Version>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Scope {
    User,

    #[default]
    Machine,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[repr(C)]
pub enum Attributes {
    #[default]
    None,

    MinVersionInclusive = 0x100,
    MaxVersionInclusive = 0x200,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Format,
    NotFound,
    NotSupported,
    RegistryError(windows::core::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Gets information about a provider.
pub fn get_provider<K>(provider_key: K, scope: Scope) -> Result<Provider>
where
    K: AsRef<str>,
{
    let key = registry::Key::open::<HKEY, PCWSTR>(
        scope.into(),
        w!("Software\\Classes\\Installer\\Dependencies"),
    )
    .map_err(map_registry_error)?;

    let _provider_key: Vec<u16> = provider_key
        .as_ref()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let _provider_key: PCWSTR = PCWSTR::from_raw(_provider_key.as_ptr());
    let key = key
        .open_subkey::<PCWSTR>(_provider_key)
        .map_err(map_registry_error)?;

    Ok(Provider::from(provider_key, &key))
}

/// Checks that there are no dependents registered for providers that are being uninstalled.
pub fn check_dependents<K>(
    provider_key: K,
    scope: Scope,
    #[allow(unused_variables)] // Prevent future breaking change; not currently used.
    attributes: Attributes,
    ignore: &Option<Vec<K>>,
) -> Result<Option<Vec<Provider>>>
where
    K: AsRef<str> + PartialEq,
{
    // Failure to open a provider or Dependents key means no dependents.
    let key = match registry::Key::open::<HKEY, PCWSTR>(
        scope.into(),
        w!("Software\\Classes\\Installer\\Dependencies"),
    ) {
        Ok(k) => k,
        Err(err) if err.code() == registry::E_FILE_NOT_FOUND => return Ok(None),
        Err(err) => return Err(Error::RegistryError(err)),
    };

    let provider_key: Vec<u16> = provider_key
        .as_ref()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let provider_key: PCWSTR = PCWSTR::from_raw(provider_key.as_ptr());
    let key = match key.open_subkey::<PCWSTR>(provider_key) {
        Ok(k) => k,
        Err(err) if err.code() == registry::E_FILE_NOT_FOUND => return Ok(None),
        Err(err) => return Err(Error::RegistryError(err)),
    };

    let key = match key.open_subkey::<PCWSTR>(w!("Dependents")) {
        Ok(k) => k,
        Err(err) if err.code() == registry::E_FILE_NOT_FOUND => return Ok(None),
        Err(err) => return Err(Error::RegistryError(err)),
    };

    Ok(Some(
        key.keys()?
            .filter_map(|k| unsafe {
                if let Some(ignore) = ignore {
                    if ignore.contains(std::mem::transmute(&k.name)) {
                        return None;
                    }
                }

                // BUGBUG: Should we check that the provider actually exists in case it didn't clean up during uninstall or was that meant for permanent packages?
                if let Ok(p) = get_provider(&k.name, scope) {
                    return Some(p);
                }

                Some(Provider {
                    key: k.name.clone(),
                    ..Default::default()
                })
            })
            .collect(),
    ))
}

impl Provider {
    fn from<K>(provider_key: K, key: &registry::Key) -> Self
    where
        K: AsRef<str>,
    {
        Provider {
            key: provider_key.as_ref().to_string(),
            id: key.value(PCWSTR::null()).and_then(|v| v.as_string()),
            name: key.value(w!("DisplayName")).and_then(|v| v.as_string()),
            version: key.value(w!("Version")).and_then(|v| v.as_version()),
        }
    }
}

impl Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            return write!(f, "{} ({})", name, &self.key);
        }
        write!(f, "{}", &self.key)
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

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Format => write!(f, "invalid format"),
            Error::NotFound => write!(f, "not found"),
            Error::NotSupported => write!(f, "not supported"),
            Error::RegistryError(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<windows::core::Error> for Error {
    fn from(value: windows::core::Error) -> Self {
        Error::RegistryError(value)
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

fn map_registry_error(err: windows::core::Error) -> Error {
    match err.code() {
        registry::E_FILE_NOT_FOUND => Error::NotFound,
        _ => Error::RegistryError(err),
    }
}
