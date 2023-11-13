// Copyright 2023 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use std::{fmt::Display, str::FromStr};

use windows::{
    core::{w, PCWSTR},
    Win32::System::Registry::HKEY,
};

mod registry;

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
    NotFound,
    NotSupported,
    RegistryError(windows::core::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn check_dependents<K>(
    provider_key: K,
    scope: Scope,
    _attributes: Attributes,
    ignore: &Option<Vec<K>>,
) -> Result<Vec<String>>
where
    K: AsRef<str> + PartialEq,
{
    let key = registry::Key::open::<HKEY, PCWSTR>(
        scope.into(),
        w!("Software\\Classes\\Installer\\Dependencies"),
    )
    .map_err(map_windows_error)?;

    let provider_key: Vec<u16> = provider_key
        .as_ref()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let provider_key: PCWSTR = PCWSTR::from_raw(provider_key.as_ptr());
    let key = key
        .open_subkey::<PCWSTR>(provider_key)
        .map_err(map_windows_error)?;

    let key = key
        .open_subkey::<PCWSTR>(w!("Dependents"))
        .map_err(map_windows_error)?;

    Ok(key
        .keys()?
        .filter_map(|k| unsafe {
            if let Some(ignore) = ignore {
                if ignore.contains(std::mem::transmute(&k.name)) {
                    return None;
                }
            }
            Some(k.name.clone())
        })
        .collect())
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotFound => write!(f, "not found"),
            Error::NotSupported => write!(f, "not supported"),
            Error::RegistryError(err) => write!(f, "{}", err),
        }
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

fn map_windows_error(err: windows::core::Error) -> Error {
    match err.code() {
        registry::E_FILE_NOT_FOUND => Error::NotFound,
        _ => Error::RegistryError(err),
    }
}
