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

use accounts::{get_key_dir, Account};
use client::BlockKey;
use client::ValidatorClient;
use jsonrpc_core::{Error, Params, Value};
use messages::seth::{
    CreateExternalAccountTxn, EvmPermissions, SethTransaction as SethTransactionPb,
    SethTransaction_TransactionType,
};
use requests::RequestHandler;
use sawtooth_sdk::messaging::stream::MessageSender;
use sawtooth_sdk::signing::secp256k1::{Secp256k1Context, Secp256k1PrivateKey};
use sawtooth_sdk::signing::Context;
use std::fs;
use transactions::SethTransaction;
use transform;

pub fn get_method_list<T>() -> Vec<(String, RequestHandler<T>)>
where
    T: MessageSender,
{
    vec![
        ("personal_listAccounts".into(), list_accounts),
        ("personal_newAccount".into(), new_account),
        ("personal_unlockAccount".into(), unlock_account),
        ("personal_importRawKey".into(), import_raw_key),
    ]
}

#[allow(needless_pass_by_value)]
/// Returns a list of loaded accounts
pub fn list_accounts<T>(_: Params, client: ValidatorClient<T>) -> Result<Value, Error>
where
    T: MessageSender,
{
    info!("personal_listAccounts");

    let accounts = client.loaded_accounts();
    let loaded_accounts = accounts.read().unwrap();

    Ok(Value::Array(
        loaded_accounts
            .iter()
            .map(|a| transform::hex_prefix(a.address()))
            .collect::<Vec<_>>(),
    ))
}

#[allow(needless_pass_by_value)]
/// Creates a new account
pub fn new_account<T>(params: Params, client: ValidatorClient<T>) -> Result<Value, Error>
where
    T: MessageSender,
{
    info!("personal_newAccount");

    let args: Vec<Option<String>> = match params.parse::<Vec<Option<String>>>() {
        Ok(t) => t,
        Err(_) => {
            return Err(Error::invalid_params(
                "Takes [password: String, moderator: String, permissions: String]",
            ));
        }
    };

    // We get Option<Option<String>> here due the possibility of a client
    // sending in an incomplete list of optional strings. Unnest into plain
    // options to fix
    let password = args.get(0).unwrap_or(&None);
    let moderator = args.get(1).unwrap_or(&None);

    let permissions = match args
        .get(2)
        .unwrap_or(&None)
        .clone()
        .map(|p| p.parse::<EvmPermissions>())
    {
        Some(Ok(perms)) => Some(perms),
        None => None,
        Some(Err(_)) => Err(Error::invalid_params("Invalid permissions value."))?,
    };

    let context = Secp256k1Context::new();

    let priv_key_hex = context
        .new_random_private_key()
        .map_err(|err| fail!("Couldn't generate key", err))?
        .as_hex();

    let priv_key = Secp256k1PrivateKey::from_hex(&priv_key_hex)
        .map_err(|err| fail!("Couldn't unbox key", err))?;

    let pem_bytes = match password {
        Some(pw) => priv_key.to_pem_with_password(pw),
        None => priv_key.to_pem(),
    }.map_err(|err| fail!("Couldn't convert key to pem string", err))?;

    let account = Account::load_from_str(&pem_bytes, password)
        .map_err(|err| fail!("Error generating key", err))?;

    let mut filename = get_key_dir().ok_or_else(Error::internal_error)?;
    filename.push(account.address());

    fs::write(filename.with_extension("pem"), pem_bytes)
        .map_err(|err| fail!("Error generating key", err))?;

    // New accounts can have a moderator that is the actual account used for sending this
    // transaction. Otherwise, the newly-created account's address is used for the transaction.
    let sender = moderator
        .clone()
        .or_else(|| Some(account.address().into()))
        .ok_or_else(|| Error::invalid_params("Invalid account specified"))?;

    // Use the nonce for the account that is actually sending the transaction
    let nonce = match client.get_account(&sender, BlockKey::Latest) {
        Ok(Some(account)) => Ok(account.nonce),
        Ok(None) => Ok(0),
        Err(err) => Err(fail!("Couldn't get account", err)),
    }?;

    // Create and send the transaction in for processing
    let mut txn = SethTransactionPb::new();
    txn.set_transaction_type(SethTransaction_TransactionType::CREATE_EXTERNAL_ACCOUNT);
    txn.set_create_external_account({
        let mut inner = CreateExternalAccountTxn::new();
        inner.set_nonce(nonce);

        // If creating an account with a moderator, we sign the transaction as from the moderator,
        // so we have to set the `to` field to the newly-created account, otherwise we'll just be
        // attempting to recreate the moderator account.
        if moderator.is_some() {
            inner.set_to(
                transform::hex_str_to_bytes(&account.address())
                    .ok_or_else(|| Error::invalid_params("Invalid moderator address!"))?,
            );
        }

        if let Some(perms) = permissions {
            inner.set_permissions(perms);
        }

        inner
    });

    client
        .send_transaction(
            &sender,
            &SethTransaction::try_from(txn).ok_or_else(Error::internal_error)?,
        ).map_err(|err| fail!("Error sending transaction", err))?;

    client
        .unlock_account(&account, Some(0))
        .map_err(|err| fail!("Couldn't unlock account", err))?;

    Ok(transform::hex_prefix(&account.address()))
}

#[allow(needless_pass_by_value)]
/// Unlocks an account, loading it from disk if necessary
pub fn unlock_account<T>(params: Params, client: ValidatorClient<T>) -> Result<Value, Error>
where
    T: MessageSender,
{
    info!("personal_unlockAccount");

    let (address, password, duration): (String, Option<String>, Option<u64>) = match params.parse()
    {
        Ok((a, p, d)) => (a, p, d),
        Err(err) => {
            warn!("Error encountered while parsing parameters: {:?}", err);
            return Ok(Value::Bool(false));
        }
    };

    Ok(Value::Bool(
        match client.unlock_address(&address, &password, duration) {
            Ok(()) => true,
            Err(err) => {
                error!("Encountered error while unlocking account: {}", err);
                false
            }
        },
    ))
}

#[allow(needless_pass_by_value)]
/// Imports a raw, hex-encoded secp256k1 key
pub fn import_raw_key<T>(params: Params, client: ValidatorClient<T>) -> Result<Value, Error>
where
    T: MessageSender,
{
    info!("personal_importRawKey");

    let usage = "Takes [key: DATA]";

    let (key, password): (String, Option<String>) =
        params.parse().map_err(|_| Error::invalid_params(usage))?;

    let priv_key = Secp256k1PrivateKey::from_hex(&key)
        .map_err(|err| fail!("Private key must be hex-encoded", err))?;
    let pem_str = match password {
        Some(ref pw) => priv_key.to_pem_with_password(&pw),
        None => priv_key.to_pem(),
    }.map_err(|err| fail!("Couldn't load key", err))?;

    let account = Account::load_from_str(&pem_str, &password)
        .map_err(|err| fail!("Error loading account from key", err))?;

    let mut filename = get_key_dir().ok_or_else(Error::internal_error)?;
    filename.push(account.address());

    fs::write(filename.with_extension("pem"), pem_str)
        .map_err(|err| fail!("Error generating key", err))?;

    // Create and send the transaction in for processing
    let mut txn = SethTransactionPb::new();
    txn.set_transaction_type(SethTransaction_TransactionType::CREATE_EXTERNAL_ACCOUNT);
    txn.set_create_external_account({
        let mut inner = CreateExternalAccountTxn::new();
        inner.set_nonce(0);

        inner
    });

    client
        .send_transaction(
            &account.address(),
            &SethTransaction::try_from(txn).ok_or_else(Error::internal_error)?,
        ).map_err(|err| fail!("Error sending transaction", err))?;

    client
        .unlock_account(&account, Some(0))
        .map_err(|err| fail!("Couldn't unlock account", err))?;

    Ok(transform::hex_prefix(&account.address()))
}
