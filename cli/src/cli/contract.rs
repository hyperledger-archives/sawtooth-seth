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

//! Manage contract accounts

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use client::Client;
use failure::Error;
use serde_json::to_string_pretty;
use tiny_keccak::keccak256;
use types::TransactionReceipt;

/// Returns Clap configuration
pub fn get_cli<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("contract")
        .about("Manages contracts in Seth")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommands(vec![
            SubCommand::with_name("call")
                .about("Calls a seth contract")
                .args(&[
                    Arg::with_name("from")
                        .short("f")
                        .long("from")
                        .takes_value(true)
                        .required(true)
                        .help("The address that the transaction will be sent from"),
                    Arg::with_name("data")
                        .short("d")
                        .long("data")
                        .takes_value(true)
                        .required(true)
                        .help(
                            "Input data to pass to contract when called; must conform to \
                             contract ABI",
                        ),
                    Arg::with_name("chaining")
                        .short("c")
                        .long("chaining-enabled")
                        .help("If true, enables contract chaining"),
                    Arg::with_name("gas")
                        .short("g")
                        .long("gas")
                        .takes_value(true)
                        .default_value(::DEFAULT_GAS)
                        .help("Gas limit for calling the contract (default: 90,000)"),
                    ::get_wait_arg(),
                    Arg::with_name("address")
                        .required(true)
                        .help("Address of contract to call"),
                ]),
            SubCommand::with_name("create")
                .about("Creates a seth contract")
                .args(&[
                    Arg::with_name("from")
                        .required(true)
                        .help("Account address used to create the contract"),
                    Arg::with_name("init")
                        .required(true)
                        .help("Initialization code to be executed on deployment"),
                    Arg::with_name("gas")
                        .short("g")
                        .long("gas")
                        .takes_value(true)
                        .default_value(::DEFAULT_GAS)
                        .help("Gas limit for creating the contract (default: {})"),
                    Arg::with_name("permissions")
                        .short("p")
                        .long("permissions")
                        .takes_value(true)
                        .help("Permissions for the new contract"),
                    ::get_wait_arg(),
                ]),
            SubCommand::with_name("list")
                .about("Lists seth contracts")
                .args(&[Arg::with_name("address")
                    .required(true)
                    .help("Address to list contracts for")]),
        ])
}

/// Handles parsing Clap CLI matches
pub fn parse_cli<'a>(matches: (&'a str, Option<&'a ArgMatches>)) -> Result<(), Error> {
    let client = &::client::Client::new()?;

    match matches {
        ("call", Some(m)) => {
            let from = m.value_of("from").expect("From address is required!");
            let address = m.value_of("address").expect("Address is required!");
            let data = m.value_of("data").expect("Data is required!");
            let chaining = m.is_present("chaining");
            let gas = m
                .value_of("gas")
                .expect("Default gas must exist!")
                .parse::<u64>()?;
            let wait = ::parse_wait_flag(m)?;

            do_call(client, from, address, data, chaining, gas, wait)?;
        }
        ("create", Some(m)) => {
            let from = m.value_of("from").expect("From address is required!");
            let init = m.value_of("init").expect("Contract value is required!");
            let gas = m
                .value_of("gas")
                .expect("Default gas must exist!")
                .parse::<u64>()?;
            let permissions = m.value_of("permissions");
            let wait = ::parse_wait_flag(m)?;

            do_create(client, from, init, gas, permissions, wait)?;
        }
        ("list", Some(m)) => {
            let address = m.value_of("address").expect("Address is required!");

            do_list(client, address)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

/// Calls a contract
pub fn do_call(
    client: &Client,
    from: &str,
    address: &str,
    data: &str,
    chaining: bool,
    gas: u64,
    wait: Option<u64>,
) -> Result<(), Error> {
    let txn_id: String = client.send_rpc_transaction(
        "eth_sendTransaction",
        &vec![json!({
            "from": format!("0x{}", from),
            "data": format!("0x{}", data),
            "to": format!("0x{}", address),
            "gas": format!("0x{:x}", gas),
            "chaining": chaining,
        })],
    )?;

    let (gas_used, retval, status) = match wait {
        Some(w) => {
            let receipt: TransactionReceipt = client.wait_for_rpc_transaction(
                "eth_getTransactionReceipt",
                &vec![txn_id.clone()],
                w,
            )?;

            (
                u64::from_str_radix(&receipt.gas_used[2..], 16)?,
                receipt.return_value[2..].to_string(),
                receipt.status[2..].to_string(),
            )
        }
        None => (0u64, "<Not retrieved>".into(), "0x0".into()),
    };

    println!(
        "{}",
        to_string_pretty(&json!({
            "From": from,
            "To": address,
            "TransactionID": txn_id,
            "GasUsed": gas_used,
            "ReturnValue": retval,
            "Status": status
        }))?
    );

    Ok(())
}

/// Creates a contract
pub fn do_create(
    client: &Client,
    from: &str,
    init: &str,
    gas: u64,
    permissions: Option<&str>,
    wait: Option<u64>,
) -> Result<(), Error> {
    let txn_id: String = client.send_rpc_transaction(
        "eth_sendTransaction",
        &vec![json!({
            "from": format!("0x{}", from),
            "data": format!("0x{}", init),
            "gas": format!("0x{:x}", gas),
            "permissions": permissions,
        })],
    )?;

    let (gas_used, retval, address) = match wait {
        Some(w) => {
            let receipt: TransactionReceipt = client.wait_for_rpc_transaction(
                "eth_getTransactionReceipt",
                &vec![txn_id.clone()],
                w,
            )?;
            (
                u64::from_str_radix(&receipt.gas_used[2..], 16)?,
                receipt.return_value[2..].to_string(),
                receipt.contract_address.to_string(),
            )
        }
        None => (0u64, "<Not retrieved>".into(), "<Not retrieved>".into()),
    };

    println!(
        "{}",
        to_string_pretty(&json!({
            "TransactionID": txn_id,
            "Address": address,
            "GasUsed": gas_used,
            "ReturnValue": retval,
        }))?
    );

    Ok(())
}

/// Lists contracts for a given alias
///
/// Stops listing contracts if it encounters an error while talking to the JSON-RPC API
pub fn do_list(client: &Client, address: &str) -> Result<(), Error> {
    let nonce: String =
        client.send_rpc_transaction("eth_getTransactionCount", &vec![address, "latest"])?;

    let contracts = (1..u64::from_str_radix(&nonce[2..], 16)?)
        .filter_map(|n| {
            let derived = derive(address.to_string(), n);

            // Attempt to get the balance to see if it's an actual account
            let result: Result<String, Error> =
                client.send_rpc_transaction("eth_getBalance", &vec![address, "latest"]);

            // Skip empty addresses, blow up on errors, and otherwise display it
            match result {
                Ok(_) => Some(derived),
                Err(_) => None,
            }
        }).collect::<Vec<_>>();

    println!("{}", to_string_pretty(&json!(contracts))?);

    Ok(())
}

// Utility functions

/// Derive a contract address from the main Account address
pub fn derive(address: String, nonce: u64) -> String {
    match nonce {
        0 => address,
        _ => {
            // Start off with the Account address
            let mut derived = address[2..]
                .chars()
                .collect::<Vec<_>>()
                .chunks(2)
                .map(|chunk| {
                    ((chunk[0].to_digit(16).expect("Got a non-hex digit!") << 4)
                        | (chunk[1].to_digit(16).expect("Got a non-hex digit!")))
                        as u8
                }).collect::<Vec<_>>();

            // Add in the bytes of the u64 nonce, big-endian style
            for i in (0..8).rev() {
                let shift = 8 * i;
                derived.push(((nonce & (0xFF << shift)) >> shift) as u8);
            }

            // And then hash to a new address
            keccak256(&derived)[..20]
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join("")
        }
    }
}
