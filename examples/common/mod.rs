// Copyright 2023 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use clap::{builder::PossibleValue, ValueEnum};

#[derive(Clone, Copy, Debug)]
pub enum Scope {
    Machine,
    User,
}

impl Default for Scope {
    fn default() -> Self {
        Self::Machine
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Machine => write!(f, "machine"),
            Self::User => write!(f, "scope"),
        }
    }
}

impl From<Scope> for wixpkgdep::Scope {
    fn from(value: Scope) -> Self {
        match value {
            Scope::Machine => Self::Machine,
            Scope::User => Self::User,
        }
    }
}

impl ValueEnum for Scope {
    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::Machine => PossibleValue::new("machine"),
            Self::User => PossibleValue::new("user"),
        })
    }

    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Machine, Self::User]
    }
}
