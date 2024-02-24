// Copyright 2023 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Format,
    NotFound,
    NotSupported,
    RegistryError(windows::core::Error),
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
