// Copyright 2023 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use clap::Parser;
use std::{collections::HashSet, error::Error};

mod common;
use common::Scope;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let Some(dependents) = wixpkgdep::check_dependents(
        &args.provider_key,
        args.scope.into(),
        Default::default(),
        args.ignored().as_ref(),
    )?
    else {
        return Ok(());
    };

    for d in dependents.iter() {
        println!("{d}");
    }

    if !dependents.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

/// Checks for dependents of a provider key.
///
/// If any dependents are found they are printed and the process terminates with exit code 1.
#[derive(Parser)]
#[command(author, version)]
struct Args {
    /// The provider key to check for dependents.
    #[arg(short = 'k', long, value_name = "KEY")]
    provider_key: String,

    /// The scope under which to check for dependents.
    #[arg(long, value_parser, default_value_t)]
    scope: Scope,

    /// Dependents to ignore.
    #[arg(long, value_name = "DEPENDENCIES")]
    ignore: Option<Vec<String>>,
}

impl Args {
    fn ignored(&self) -> Option<HashSet<String>> {
        self.ignore
            .as_ref()
            .map(|v| HashSet::from_iter(v.iter().cloned()))
    }
}
