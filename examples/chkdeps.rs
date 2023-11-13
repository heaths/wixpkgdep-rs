// Copyright 2023 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use clap::Parser;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let dependents = wixpkgdep::check_dependents(
        args.provider_key,
        args.scope,
        Default::default(),
        &args.ignore,
    )?;

    for d in dependents.iter() {
        println!("{d}");
    }

    if !dependents.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

#[derive(Parser)]
#[command(author, version, about = "Gets dependents of a provider key.")]
struct Args {
    /// The provider key to check for dependents.
    #[arg(short = 'k', long, value_name = "KEY")]
    provider_key: String,

    /// The scope under which to check for dependents.
    #[arg(long, value_parser, default_value_t)]
    scope: wixpkgdep::Scope,

    /// Dependents to ignore.
    #[arg(long, value_name = "DEPENDENCIES")]
    ignore: Option<Vec<String>>,
}
