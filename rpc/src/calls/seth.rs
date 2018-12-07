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

use client::{BlockKey, ValidatorClient};
use jsonrpc_core::{Error, Params, Value};
use messages::seth::EvmPermissions;
use messages::seth::SetPermissionsTxn;
use messages::seth::SethTransaction as SethTransactionPb;
use messages::seth::SethTransaction_TransactionType;
use requests::RequestHandler;
use sawtooth_sdk::messaging::stream::MessageSender;
use std::time::{SystemTime, UNIX_EPOCH};
use transactions::SethTransaction;
use transform;

pub fn get_method_list<T>() -> Vec<(String, RequestHandler<T>)>
where
    T: MessageSender,
{
    vec![
        ("seth_getPermissions".into(), get_permissions),
        ("seth_setPermissions".into(), set_permissions),
    ]
}

#[allow(needless_pass_by_value)]
pub fn get_permissions<T>(params: Params, client: ValidatorClient<T>) -> Result<Value, Error>
where
    T: MessageSender,
{
    info!("seth_getPermissions");

    let usage = "Takes [address: ADDRESS]";

    let (address,): (String,) = params.parse().map_err(|_| Error::invalid_params(usage))?;

    let account = client
        .get_account(&address, BlockKey::Latest)
        .map_err(|err| fail!("Couldn't get key", err))?;

    match account {
        Some(a) => Ok(Value::String(format!("{}", a.get_permissions()))),
        None => Ok(Value::Null),
    }
}

#[allow(needless_pass_by_value)]
pub fn set_permissions<T>(params: Params, client: ValidatorClient<T>) -> Result<Value, Error>
where
    T: MessageSender,
{
    info!("seth_setPermissions");

    let usage = "Takes [address: ADDRESS, permissions: DATA]";

    let (address, permissions): (String, String) =
        params.parse().map_err(|_| Error::invalid_params(usage))?;

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| fail!("Time has gone backwards", err))?
        .as_secs();

    // Create and send the transaction in for processing
    let mut txn = SethTransactionPb::new();
    txn.set_transaction_type(SethTransaction_TransactionType::SET_PERMISSIONS);
    txn.set_set_permissions({
        let mut inner = SetPermissionsTxn::new();
        inner.set_nonce(nonce);
        inner.set_to(
            transform::hex_str_to_bytes(&address).ok_or_else(|| Error::invalid_params(usage))?,
        );
        inner.set_permissions(
            permissions
                .parse::<EvmPermissions>()
                .map_err(|err| fail!("Couldn't parse permissions", err))?,
        );
        inner
    });

    client
        .send_transaction(
            &client
                .unlocked_account()
                .ok_or_else(|| fail!("Couldn't unlock account"))?
                .public_key(),
            &SethTransaction::try_from(txn).ok_or_else(|| fail!("Couldn't create transaction"))?,
        )
        .map_err(|err| fail!("Couldn't send transaction", err))?;

    Ok(Value::Bool(true))
}
