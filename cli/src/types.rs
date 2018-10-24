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

//! Common Seth types

/// Seth transaction log
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionLog {
    pub removed: bool,
    pub log_index: String,
    pub transaction_index: String,
    pub transaction_hash: String,
    pub block_hash: String,
    pub block_number: String,
    pub address: String,
    pub data: String,
    pub topics: Vec<String>,
}

/// Seth transaction receipt
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    pub transaction_hash: String,
    pub transaction_index: String,
    pub block_hash: String,
    pub block_number: String,
    pub cumulative_gas_used: String,
    pub gas_used: String,
    pub contract_address: String,
    pub logs: Vec<TransactionLog>,
    pub return_value: String,
    pub status: String,
}
