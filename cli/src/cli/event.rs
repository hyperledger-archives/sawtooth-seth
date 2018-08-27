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

//! Show events for a transaction

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use client::Client;
use failure::Error;
use serde_json::to_string_pretty;
use types::TransactionReceipt;

/// Returns Clap configuration
pub fn get_cli<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("event")
        .about("Manages seth events")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommands(vec![
            SubCommand::with_name("list")
                .about("Lists seth events")
                .args(&[Arg::with_name("txn-id")
                    .required(true)
                    .help("Which transaction to show events for")]),
        ])
}

/// Handles parsing Clap CLI matches
pub fn parse_cli<'a>(matches: (&'a str, Option<&'a ArgMatches>)) -> Result<(), Error> {
    let client = &::client::Client::new()?;

    match matches {
        ("list", Some(m)) => {
            let txn_id = m.value_of("txn-id").expect("Transaction ID is required!");

            do_list(client, txn_id)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

/// Lists events for the given transaction
pub fn do_list(client: &Client, txn_id: &str) -> Result<(), Error> {
    let receipt: TransactionReceipt =
        client.send_rpc_transaction("eth_getTransactionReceipt", &vec![format!("0x{}", txn_id)])?;

    println!("{}", to_string_pretty(&json!(receipt.logs))?);

    Ok(())
}
