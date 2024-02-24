// Copyright 2023 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use crate::registry::{Data, Key};
use crate::version::Version;
use crate::{Attributes, Result, Scope};
use std::{collections::HashSet, fmt::Display, hash};
use windows::core::{w, PCWSTR};
use windows::Win32::System::Registry::HKEY;

#[derive(Debug, Default, Clone, Eq)]
pub struct Dependency {
    /// Provider key that uniquely identifies the dependency.
    pub key: String,
}

impl Dependency {
    pub(crate) fn new(provider_key: impl Into<String>) -> Self {
        Dependency {
            key: provider_key.into(),
        }
    }
}

impl Display for Dependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.key)
    }
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool {
        self.key.to_uppercase().eq(&other.key.to_uppercase())
    }
}

impl hash::Hash for Dependency {
    // cspell:ignore Hasher
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.key.to_uppercase().hash(state)
    }
}

#[derive(Debug, Default, Clone, Eq)]
pub struct Provider {
    /// Provider key that uniquely identifies the provider.
    pub key: String,

    /// Optional display name of the provider.
    pub name: String,

    /// Version of the provider.
    pub version: Version,

    /// Optional identifier of the package for an external system e.g., a ProductCode for a Windows Installer package.
    #[allow(dead_code)] // TODO
    pub id: Option<String>,

    /// Optional attributes used when checking dependencies.
    pub attributes: Option<Attributes>,
}

impl Provider {
    pub(crate) fn from(provider_key: impl Into<String>, key: &Key) -> crate::Result<Self> {
        // Equivalent to deputil:DepGetProviderInformation.
        Ok(Provider {
            key: provider_key.into(),
            name: key.value(w!("DisplayName"))?.to_string()?,
            version: key.value(w!("Version"))?.to_version()?,
            id: key.value(PCWSTR::null())?.to_string().ok(),
            ..Default::default()
        })
    }

    /// Checks that there are no dependents registered for the current provider that are being uninstalled.
    pub fn check_dependents<K>(
        &self,
        scope: Scope,
        #[allow(unused_variables)] // Prevent future breaking change; not currently used.
        attributes: Option<Attributes>,
        ignore: Option<&HashSet<String>>,
    ) -> Result<Option<Vec<Dependency>>> {
        crate::check_dependents(&self.key, scope, attributes, ignore)
    }

    /// Registers the [`Provider`].
    pub fn register(&self, scope: Scope) -> crate::Result<()> {
        // Equivalent to deputil:DepRegisterDependency.
        let key = Key::create::<HKEY, PCWSTR>(scope.into(), crate::ROOT_KEY)?;

        let provider_key = crate::to_pcwstr(&self.key);
        let key = key.create_subkey(provider_key)?;

        key.set_value(Some(w!("DisplayName")), Data::String(self.name.to_string()))?;
        key.set_value(Some(w!("Version")), Data::String(self.version.to_string()))?;
        if let Some(id) = &self.id {
            key.set_value(None, Data::String(id.to_string()))?;
        }
        if let Some(attributes) = self.attributes {
            key.set_value(Some(w!("Attributes")), Data::DWord(attributes as u32))?;
        }

        Ok(())
    }
}

impl Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.name.is_empty() {
            return write!(f, "{} ({})", &self.name, &self.key);
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
                name: "display".to_string(),
                ..Default::default()
            }
            .to_string(),
            "display (test)"
        );
    }
}
