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

//! Show receipts

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use client::Client;
use failure::Error;
use serde_json::to_string_pretty;
use types::TransactionReceipt;

/// Returns Clap configuration
pub fn get_cli<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("receipt")
        .about("Manages seth receipts")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommands(vec![SubCommand::with_name("show")
            .about("Manages seth receipts")
            .args(&[Arg::with_name("txn-id")
                .required(true)
                .help("Transaction ID of receipt to show")])])
}

/// Handles parsing Clap CLI matches
pub fn parse_cli<'a>(matches: (&'a str, Option<&'a ArgMatches>)) -> Result<(), Error> {
    let client = &::client::Client::new()?;

    match matches {
        ("show", Some(m)) => {
            let txn_id = m.value_of("txn-id").expect("Transaction ID is required!");

            do_show(client, txn_id)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

/// Shows the receipt for the given transaction
pub fn do_show(client: &Client, txn_id: &str) -> Result<(), Error> {
    let receipt: TransactionReceipt =
        client.send_rpc_transaction("eth_getTransactionReceipt", &vec![format!("0x{}", txn_id)])?;

    println!(
        "{}",
        to_string_pretty(&json!({
            "From": receipt.from,
            "To": receipt.to,
            "GasUsed": u64::from_str_radix(&receipt.gas_used[2..], 16)?,
            "Address": receipt.contract_address,
            "ReturnValue": receipt.return_value,
            "Status": receipt.status,
        }))?
    );

    Ok(())
}
