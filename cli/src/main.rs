/*
 * Copyright 2018 Intel Corporation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * ------------------------------------------------------------------------------
 */

//! Command-line interface for interacting with the Seth JSON-RPC API
//!
//! Example usage:
//!
//!     // Create config file
//!     seth config init http://seth-rpc:8080/
//!
//!     // Can also specify the URL with an env var
//!     SETH_URL=http://seth-rpc:8080/ seth command do
//!
//! For more options, see `seth --help`

#[macro_use]
extern crate clap;
extern crate config;
extern crate dirs;
#[macro_use]
extern crate failure;
extern crate jsonrpc_core;
extern crate reqwest;
extern crate sawtooth_sdk;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate time;
extern crate tiny_keccak;
extern crate toml;

pub mod cli;
pub mod client;
pub mod types;

use clap::{App, AppSettings, Arg, ArgMatches};
use failure::Error;

const DEFAULT_GAS: &str = "90000";

/// Used in a bunch of places with the same setup, so just define it once here
fn get_wait_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("wait")
        .short("w")
        .long("wait")
        .takes_value(true)
        .min_values(0)
        .max_values(1)
        .help(
            "Number of seconds that the client will wait for transaction to be committed. \
             If flag is passed, value in seconds (or a default of 60) is used. Otherwise, the \
             client will not wait",
        )
}

/// Parses an optional wait argument that can be used as a simple flag, or an option
/// Example CLI usages:
///   <elided>      // If left out completely, returns None
///   -w            // If passed in as just a flag, returns Option<60>
///   -w 42         // If passed in with a value, returns Option<value>
/// The latter two cases are also coerced into the correct type
fn parse_wait_flag(matches: &ArgMatches) -> Result<Option<u64>, Error> {
    if matches.is_present("wait") {
        let val = matches.value_of("wait");
        match val.or(Some("60")).and_then(|w| w.parse::<u64>().ok()) {
            Some(arg) => Ok(Some(arg)),
            None => Err(format_err!(
                "Bad value for wait: `{}`",
                val.expect("Wait must exist!")
            ))?,
        }
    } else {
        Ok(None)
    }
}

fn run() -> Result<(), Error> {
    // Define CLI
    let matches = App::new("seth")
        .author(crate_authors!())
        .version(crate_version!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommands(vec![
            cli::account::get_cli(),
            cli::config::get_cli(),
            cli::contract::get_cli(),
            cli::event::get_cli(),
            cli::permissions::get_cli(),
            cli::receipt::get_cli(),
        ])
        .get_matches();

    match matches.subcommand() {
        ("account", Some(am)) => cli::account::parse_cli(am.subcommand())?,
        ("config", Some(sm)) => cli::config::parse_cli(sm.subcommand())?,
        ("contract", Some(cm)) => cli::contract::parse_cli(cm.subcommand())?,
        ("event", Some(em)) => cli::event::parse_cli(em.subcommand())?,
        ("permissions", Some(pm)) => cli::permissions::parse_cli(pm.subcommand())?,
        ("receipt", Some(rm)) => cli::receipt::parse_cli(rm.subcommand())?,
        _ => unreachable!(),
    }

    Ok(())
}

fn main() {
    // Attempt to run command, and print out any errors encountered
    if let Err(e) = run() {
        eprint!("Error: {}", e);
        let mut e = e.as_fail();
        while let Some(cause) = e.cause() {
            eprint!(", {}", cause);
            e = cause;
        }
        eprintln!();
        std::process::exit(1);
    }
}
