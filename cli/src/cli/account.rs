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

//! Manage externally owned accounts

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use client::Client;
use failure::Error;
use serde_json::to_string_pretty;
use std::fs::File;
use std::io::Read;
use std::str::from_utf8;

/// Returns Clap configuration
pub fn get_cli<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("account")
        .about("Creates a seth account")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommands(vec![
            SubCommand::with_name("create")
                .about("Creates a seth account")
                .args(&[
                    Arg::with_name("pass-file")
                        .long("pass-file")
                        .takes_value(true)
                        .help("Path to file containing password to encrypt key with"),
                    Arg::with_name("moderator")
                        .short("m")
                        .long("moderator")
                        .takes_value(true)
                        .help("Alias of another account to be used to create the account"),
                    Arg::with_name("permissions")
                        .short("p")
                        .long("permissions")
                        .takes_value(true)
                        .help(
                            "Permissions for new account; see 'seth permissions -h' for more info",
                        ),
                ]),
            SubCommand::with_name("unlock")
                .about("Unlocks a seth account")
                .args(&[
                    Arg::with_name("address").help("The account address to unlock"),
                    Arg::with_name("pass-file")
                        .long("pass-file")
                        .takes_value(true)
                        .help("Path to file containing password to unlock account with"),
                    Arg::with_name("duration")
                        .short("d")
                        .long("duration")
                        .takes_value(true)
                        .default_value("300")
                        .help("How long to unlock the account for"),
                ]),
            SubCommand::with_name("import")
                .about("Imports a seth account")
                .args(&[
                    Arg::with_name("key-file")
                        .required(true)
                        .help("Path to the file that contains the private key to import"),
                    Arg::with_name("pass-file")
                        .short("p")
                        .long("pass-file")
                        .takes_value(true)
                        .help("Path to file containing password to encrypt key with"),
                ]),
            SubCommand::with_name("list").about("Lists seth accounts"),
        ])
}

/// Handles parsing Clap CLI matches
pub fn parse_cli<'a>(matches: (&'a str, Option<&'a ArgMatches>)) -> Result<(), Error> {
    let client = &::client::Client::new()?;

    match matches {
        ("create", Some(m)) => {
            let pass_file = m.value_of("pass-file");
            let moderator = m.value_of("moderator");
            let permissions = m.value_of("permissions");

            do_create(client, pass_file, moderator, permissions)?;
        }
        ("unlock", Some(m)) => {
            let address = m.value_of("address").expect("Address is required!");
            let pass_file = m.value_of("pass-file");
            let duration = match m.value_of("duration") {
                Some(d) => Some(d.parse::<u64>()?),
                None => None,
            };

            do_unlock(client, address, pass_file, duration)?;
        }
        ("import", Some(m)) => {
            let key_file = m.value_of("key-file").expect("Key file path is required!");
            let pass_file = m.value_of("pass-file");

            do_import(client, key_file, pass_file)?;
        }
        ("list", Some(_)) => {
            do_list(&client)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

/// Creates a new account
///
/// Generates a new random public/private keypair and stores it in the RPC service's
/// account store, then creates a state entry in Seth for that account. Prints out the
/// new account's address.
///
/// If a moderator is specified, the transaction is signed by the existing moderator
/// account, and the new account is listed in the `to` field of the Seth transaction.
pub fn do_create(
    client: &Client,
    pass_file: Option<&str>,
    moderator: Option<&str>,
    permissions: Option<&str>,
) -> Result<(), Error> {
    let password = match pass_file {
        Some(pf) => {
            let mut file = File::open(&pf)?;
            let mut password = String::new();
            file.read_to_string(&mut password)?;
            Some(password.trim().to_string())
        }
        None => None,
    };

    let account_id: String = client.send_rpc_transaction(
        "personal_newAccount",
        &json!((password, moderator, permissions)),
    )?;

    println!("\"{}\"", account_id);

    Ok(())
}

/// Unlocks an existing account
///
/// When the RPC service first boots up, it will by default not have any accounts loaded
/// or unlocked. You can start the RPC service with `--unlock` to load and unlock the
/// accounts you need, or call `seth account unlock` to load and unlock a particular
/// account.
pub fn do_unlock(
    client: &Client,
    address: &str,
    pass_file: Option<&str>,
    duration: Option<u64>,
) -> Result<(), Error> {
    let password = match pass_file {
        Some(pf) => {
            let mut file = File::open(&pf)?;
            let mut password = String::new();
            file.read_to_string(&mut password)?;
            Some(password.trim().to_string())
        }
        None => None,
    };

    let result: bool = client.send_rpc_transaction(
        "personal_unlockAccount",
        &json!((address, password, duration)),
    )?;

    println!("{}", result);

    Ok(())
}

/// Imports a key into the RPC service's account store
///
/// Optionally encrypts imported key
pub fn do_import(client: &Client, key_file: &str, pass_file: Option<&str>) -> Result<(), Error> {
    let mut file = File::open(&key_file)?;
    let mut key = vec![];
    file.read_to_end(&mut key)?;

    let password = match pass_file {
        Some(pf) => {
            let mut file = File::open(&pf)?;
            let mut password = String::new();
            file.read_to_string(&mut password)?;
            Some(password.trim().to_string())
        }
        None => None,
    };

    let account_id: String = client.send_rpc_transaction(
        "personal_importRawKey",
        &json!([from_utf8(&key)?, password]),
    )?;

    println!("\"{}\"", account_id);

    Ok(())
}

/// Lists all loaded accounts
pub fn do_list(client: &Client) -> Result<(), Error> {
    let result: Vec<String> = client.send_rpc_transaction("personal_listAccounts", &json!([]))?;

    println!("{}", to_string_pretty(&json!(result))?);

    Ok(())
}
