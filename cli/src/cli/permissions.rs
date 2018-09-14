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

//! Set and get permissions for an account

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use client::Client;
use failure::Error;

/// Returns Clap configuration
pub fn get_cli<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("permissions")
        .about("Manages seth permissions")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommands(vec![
            SubCommand::with_name("set")
                .about("Sets seth permissions")
                .args(&[
                    Arg::with_name("address")
                        .short("a")
                        .long("address")
                        .takes_value(true)
                        .required(true)
                        .help(
                            "Address of account whose permissions are being changed; \
                             'global' may be used to refer to the zero address",
                        ),
                    Arg::with_name("permissions")
                        .short("p")
                        .long("permissions")
                        .takes_value(true)
                        .required(true)
                        .help("Permissions to set for the given address"),
                ]),
            SubCommand::with_name("get")
                .about("Gets seth permissions")
                .args(&[Arg::with_name("address")
                    .short("a")
                    .long("address")
                    .takes_value(true)
                    .required(true)
                    .help("Address that permissions are being retrieved for")]),
        ])
}

/// Handles parsing Clap CLI matches
pub fn parse_cli<'a>(matches: (&'a str, Option<&'a ArgMatches>)) -> Result<(), Error> {
    let client = &::client::Client::new()?;

    match matches {
        ("set", Some(m)) => {
            let address = m.value_of("address").expect("Address is required!");
            let permissions = m
                .value_of("permissions")
                .expect("Permissions are required!");

            do_set(client, address, permissions)?;
        }
        ("get", Some(m)) => {
            let address = m.value_of("address").expect("Address is required!");

            do_get(client, address)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

/// Sets the permissions for the given address
///
/// Accepts the string `"global"` to refer to the `"000...000"` address.
pub fn do_set(client: &Client, address: &str, permissions: &str) -> Result<(), Error> {
    let address = match address {
        "global" => "0000000000000000000000000000000000000000",
        _ => address,
    };

    let result: bool =
        client.send_rpc_transaction("seth_setPermissions", &json!(vec![address, permissions]))?;

    println!("{}", result);

    Ok(())
}

/// Gets the permissions for the given address
///
/// Accepts the string `"global"` to refer to the `"000...000"` address.
pub fn do_get(client: &Client, address: &str) -> Result<(), Error> {
    let address = match address {
        "global" => "0000000000000000000000000000000000000000",
        _ => address,
    };

    let result: String =
        client.send_rpc_transaction("seth_getPermissions", &json!(vec![address]))?;

    println!("\"{}\"", result);

    Ok(())
}
