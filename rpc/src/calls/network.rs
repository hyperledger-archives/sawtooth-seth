/*
 * Copyright 2017 Intel Corporation
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

use jsonrpc_core::{Error, Params, Value};

use client::ValidatorClient;
use requests::RequestHandler;

use sawtooth_sdk::messaging::stream::MessageSender;

const SAWTOOTH_NET_VERSION: &str = "19";

pub fn get_method_list<T>() -> Vec<(String, RequestHandler<T>)>
where
    T: MessageSender,
{
    vec![
        ("net_version".into(), version),
        ("net_peerCount".into(), peer_count),
        ("net_listening".into(), listening),
    ]
}

// Version refers to the particular network this JSON-RPC client is connected to
pub fn version<T>(_params: Params, _client: ValidatorClient<T>) -> Result<Value, Error>
where
    T: MessageSender,
{
    info!("net_version");
    Ok(Value::String(String::from(SAWTOOTH_NET_VERSION)))
}

// Return the number of actual Sawtooth peers
pub fn peer_count<T>(_params: Params, client: ValidatorClient<T>) -> Result<Value, Error>
where
    T: MessageSender,
{
    info!("net_peerCount");
    let n = match client.get_peers() {
        Err(_) => 0,
        Ok(n) => n,
    };

    Ok(Value::String(format!("{:#x}", n)))
}

// Return whether we are listening for connections, which is always true
pub fn listening<T>(_params: Params, _client: ValidatorClient<T>) -> Result<Value, Error>
where
    T: MessageSender,
{
    info!("net_listening");
    Ok(Value::Bool(true))
}
