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

//! Client for talking to the REST and JSON-RPC APIs

#![allow(unknown_lints)]

use config::{Config, Environment, File};
use dirs;
use failure::Error;
use jsonrpc_core::Value;
use reqwest;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::thread::sleep;
use std::time::Duration;
use time;

/// Client settings
#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub url: String,
}

/// An error returned by the JSON-RPC API
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonRpcError {
    code: i64,
    message: String,
}

/// A response from the JSON-RPC API
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonRpcResponse<R> {
    id: u64,
    jsonrpc: String,
    result: Option<R>,
    error: Option<JsonRpcError>,
}

/// Represents a client that the CLI uses to talk to any backends
///
/// Talks to both the Seth JSON-RPC API and the general Sawtooth REST API as necessary
#[derive(Debug)]
pub struct Client {
    pub url: String,
}

impl Client {
    /// Creates a new Client
    ///
    /// Loads the JSON-RPC API URL from a config file or environment variable.
    pub fn new() -> Result<Client, Error> {
        let mut settings_file =
            dirs::home_dir().ok_or_else(|| format_err!("Couldn't find home directory!"))?;

        settings_file.push(".sawtooth");
        settings_file.push("seth-config.toml");

        let mut s = Config::new();

        if settings_file.exists() {
            s.merge(File::with_name(&settings_file.to_string_lossy()))?;
        }

        s.merge(Environment::with_prefix("SETH"))?;

        s.set_default("url", "http://seth-rpc:3030/")?;

        let settings: Settings = s.try_into()?;

        Ok(Client { url: settings.url })
    }

    /// Convenience function that creates an RPC URL + path
    ///
    /// Handles stuff like stripping double slashes from e.g. `"http://foo/" + "/bar"`
    fn get_path(&self, path: &str) -> Result<String, Error> {
        Ok(reqwest::Url::parse(&self.url)?
            .join(path)?
            .as_str()
            .to_owned())
    }

    /// Sends a transaction to the JSON-RPC API
    ///
    /// Accepts `params` in the normal form of `Vec<String>`, but also as anything that
    /// can be `Into`'ed a json `Value`, such as a `String` or result from `json!()`.
    ///
    /// Returns anything that can be deserialized. In practice this means that you'll need
    /// type annotations when calling this function:
    ///
    ///     let result: bool = client.send_rpc_transaction("eth_foo", &vec![])?;
    pub fn send_rpc_transaction<V, D>(&self, method: &str, params: &V) -> Result<D, Error>
    where
        V: Into<Value> + Serialize,
        D: DeserializeOwned,
    {
        let client = reqwest::Client::new();

        let mut response = client
            .post(&self.get_path("")?)
            .json(&json!({
                "id": time::precise_time_ns(),
                "jsonrpc": "2.0",
                "method": method,
                "params": params
            })).send()?;

        let body: JsonRpcResponse<D> = response.json()?;

        match (body.result, body.error) {
            (Some(_), Some(_)) => Err(format_err!("Got both a result and an error!"))?,
            (Some(res), None) => Ok(res),
            (None, Some(err)) => Err(format_err!("{:?}", err)),
            (None, None) => Err(format_err!("Got an empty response!"))?,
        }
    }

    /// Sends a transaction to the JSON-RPC API and waits for a successful response
    ///
    /// Useful for sending `eth_getReceipt` transactions, since validating a transaction
    /// that you want a receipt for may take some time. Returns an error after `wait` seconds.
    pub fn wait_for_rpc_transaction<V, D>(
        &self,
        method: &str,
        params: &V,
        wait: u64,
    ) -> Result<D, Error>
    where
        V: Into<Value> + Serialize,
        D: DeserializeOwned,
    {
        for _ in 0..wait {
            match self.send_rpc_transaction(method, params) {
                Ok(result) => return Ok(result),
                Err(_) => sleep(Duration::new(1, 0)),
            }
        }

        Err(format_err!("Got a null response from the RPC server!"))
    }
}
