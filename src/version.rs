// Copyright 2023 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use crate::Error;
use std::cmp::Ordering;
use std::fmt::Display;

/// A comparable version containing major.minor.build.revision fields.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Version {
    major: u16,
    minor: u16,
    build: u16,
    revision: u16,
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return Some(ord),
        }

        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return Some(ord),
        }

        match self.build.cmp(&other.build) {
            Ordering::Equal => {}
            ord => return Some(ord),
        }

        Some(self.revision.cmp(&other.revision))
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.major, self.minor, self.build, self.revision
        )
    }
}

impl From<[u16; 4]> for Version {
    fn from(value: [u16; 4]) -> Self {
        Version {
            major: value[0],
            minor: value[1],
            build: value[2],
            revision: value[3],
        }
    }
}

impl From<u64> for Version {
    fn from(value: u64) -> Self {
        Version {
            major: ((value >> 48) as u16),
            minor: ((value >> 32) as u16),
            build: ((value >> 16) as u16),
            revision: (value as u16),
        }
    }
}

impl From<Version> for u64 {
    fn from(value: Version) -> u64 {
        (value.major as u64) << 48
            | (value.minor as u64) << 32
            | (value.build as u64) << 16
            | value.revision as u64
    }
}

impl TryFrom<String> for Version {
    type Error = crate::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
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
}
