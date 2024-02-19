// Copyright 2023 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use crate::registry::Key;
use crate::version::Version;
use crate::{Attributes, Result, Scope};
use std::{collections::HashSet, fmt::Display, hash};
use windows::core::{w, PCWSTR};

#[derive(Debug, Default, Clone, Eq)]
pub struct Provider {
    /// Provider key that uniquely identifies the provider.
    pub key: String,

    /// Optional identifier of the package for an external system e.g., a ProductCode for a Windows Installer package.
    #[allow(dead_code)] // TODO
    pub id: Option<String>,

    /// Optional display name of the provider.
    pub name: Option<String>,

    /// Optional version of the provider.
    pub version: Option<Version>,
}

impl Provider {
    pub(crate) fn new(provider_key: impl Into<String>) -> Self {
        Provider {
            key: provider_key.into(),
            ..Default::default()
        }
    }

    pub(crate) fn from(provider_key: impl Into<String>, key: &Key) -> Self {
        // Equivalent to deputil:DepGetProviderInformation.
        Provider {
            key: provider_key.into(),
            id: key.value(PCWSTR::null()).and_then(|v| v.as_string()),
            name: key.value(w!("DisplayName")).and_then(|v| v.as_string()),
            version: key.value(w!("Version")).and_then(|v| v.as_version()),
        }
    }

    /// Checks that there are no dependents registered for the current provider that are being uninstalled.
    pub fn check_dependents<K>(
        &self,
        scope: Scope,
        #[allow(unused_variables)] // Prevent future breaking change; not currently used.
        attributes: Option<Attributes>,
        ignore: Option<&HashSet<String>>,
    ) -> Result<Option<Vec<Provider>>> {
        crate::check_dependents(&self.key, scope, attributes, ignore)
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

impl PartialEq for Provider {
    fn eq(&self, other: &Self) -> bool {
        self.key.to_uppercase().eq(&other.key.to_uppercase())
    }
}

impl hash::Hash for Provider {
    // cspell:ignore Hasher
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.key.to_uppercase().hash(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    #[test]
    fn provider_equivalency() {
        assert_eq!(Provider::new("test"), Provider::new("test"));
        assert_eq!(hash(&Provider::new("test")), hash(&Provider::new("test")));
    }

    #[test]
    fn provider_equivalency_case_insensitive() {
        assert_eq!(Provider::new("test"), Provider::new("TEST"));
        assert_eq!(hash(&Provider::new("test")), hash(&Provider::new("TEST")));
    }

    #[test]
    fn provider_fmt() {
        assert_eq!(
            Provider {
                key: "test".to_string(),
                ..Default::default()
            }
            .to_string(),
            "test"
        );
        assert_eq!(
            Provider {
                key: "test".to_string(),
                name: Some("display".to_string()),
                ..Default::default()
            }
            .to_string(),
            "display (test)"
        );
    }

    fn hash<T: Hash>(t: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        t.hash(&mut hasher);
        hasher.finish()
    }
}
