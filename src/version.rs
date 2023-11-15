// Copyright 2023 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use crate::Error;
use std::fmt::Display;

/// A comparable version containing major.minor.build.revision fields.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(u64);

impl Version {
    /// Gets the major version number.
    pub fn major(&self) -> u16 {
        (self.0 >> 48) as u16
    }

    /// Gets the minor version number.
    pub fn minor(&self) -> u16 {
        (self.0 >> 32) as u16
    }

    /// Gets the build version number.
    pub fn build(&self) -> u16 {
        (self.0 >> 16) as u16
    }

    /// Gets the revision version number.
    pub fn revision(&self) -> u16 {
        self.0 as u16
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.major(),
            self.minor(),
            self.build(),
            self.revision()
        )
    }
}

impl From<[u16; 4]> for Version {
    fn from(value: [u16; 4]) -> Self {
        Version(
            (value[0] as u64) << 48
                | (value[1] as u64) << 32
                | (value[2] as u64) << 16
                | value[3] as u64,
        )
    }
}

impl From<u64> for Version {
    fn from(value: u64) -> Self {
        Version(value)
    }
}

impl From<Version> for u64 {
    fn from(value: Version) -> u64 {
        value.0
    }
}

impl TryFrom<String> for Version {
    type Error = crate::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Version::try_from(value.as_ref())
    }
}

impl TryFrom<&str> for Version {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim_start_matches(|c| c == 'v' || c == 'V');

        let mut fields = [0u16; 4];

        for (i, part) in value.split('.').enumerate() {
            if i >= fields.len() {
                return Err(Error::Format);
            }

            let field = part.parse::<u16>().map_err(|_| Error::Format)?;
            fields[i] = field;
        }

        Ok(Version::from(fields))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_to_string() {
        assert_eq!("1.2.3.4", Version::from([1, 2, 3, 4]).to_string());
    }

    #[test]
    fn version_from_u64() {
        assert_eq!(
            Version::from(281483566841860u64),
            Version::from([1, 2, 3, 4])
        );
    }

    #[test]
    fn version_into_u64() {
        assert_eq!(281483566841860u64, Version::from([1, 2, 3, 4]).into());
    }

    #[test]
    fn version_partial_cmp() {
        assert!(Version::from([1, 2, 3, 4]) == Version::from([1, 2, 3, 4]));
        assert!(Version::from([1, 0, 0, 0]) < Version::from([1, 1, 0, 0]));
        assert!(Version::from([1, 1, 0, 0]) > Version::from([1, 0, 0, 0]));
        assert!(Version::from([1, 2, 0, 0]) <= Version::from([1, 2, 3, 0]));
        assert!(Version::from([1, 2, 3, 0]) >= Version::from([1, 2, 0, 0]));
    }

    #[test]
    fn version_try_from_str_ok() {
        assert_eq!(Version::try_from("1").unwrap(), Version::from([1, 0, 0, 0]));
        assert_eq!(
            Version::try_from("1.2").unwrap(),
            Version::from([1, 2, 0, 0])
        );
        assert_eq!(
            Version::try_from("1.2.3").unwrap(),
            Version::from([1, 2, 3, 0])
        );
        assert_eq!(
            Version::try_from("1.2.3.4").unwrap(),
            Version::from([1, 2, 3, 4])
        );
    }

    #[test]
    fn version_try_from_str_err_format() {
        assert_eq!(
            Version::try_from("test".to_string()).unwrap_err(),
            Error::Format
        );
    }

    #[test]
    fn version_try_from_str_err_too_many() {
        assert_eq!(
            Version::try_from("1.2.3.4.5".to_string()).unwrap_err(),
            Error::Format
        );
    }

    #[test]
    fn version_try_from_string_ok() {
        assert_eq!(
            Version::try_from("1".to_string()).unwrap(),
            Version::from([1, 0, 0, 0])
        );
        assert_eq!(
            Version::try_from("1.2".to_string()).unwrap(),
            Version::from([1, 2, 0, 0])
        );
        assert_eq!(
            Version::try_from("1.2.3".to_string()).unwrap(),
            Version::from([1, 2, 3, 0])
        );
        assert_eq!(
            Version::try_from("1.2.3.4".to_string()).unwrap(),
            Version::from([1, 2, 3, 4])
        );
    }

    #[test]
    fn version_try_from_prefix_string_ok() {
        assert_eq!(
            Version::try_from("v1.2.3.4".to_string()).unwrap(),
            Version::from([1, 2, 3, 4])
        );
        assert_eq!(
            Version::try_from("V1.2.3.4".to_string()).unwrap(),
            Version::from([1, 2, 3, 4])
        );
    }

    #[test]
    fn version_try_from_string_err_format() {
        assert_eq!(
            Version::try_from("test".to_string()).unwrap_err(),
            Error::Format
        );
    }

    #[test]
    fn version_try_from_string_err_too_many() {
        assert_eq!(
            Version::try_from("1.2.3.4.5".to_string()).unwrap_err(),
            Error::Format
        );
    }

    #[test]
    fn version_properties() {
        let version = Version::from([1, 2, 3, 4]);
        assert_eq!(version.major(), 1);
        assert_eq!(version.minor(), 2);
        assert_eq!(version.build(), 3);
        assert_eq!(version.revision(), 4);
    }
}
