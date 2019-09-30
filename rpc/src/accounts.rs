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

use dirs::home_dir;
use sawtooth_sdk::signing::secp256k1::Secp256k1PrivateKey;
use sawtooth_sdk::signing::Error as SigningError;
use sawtooth_sdk::signing::{create_context, PrivateKey};
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs::File;
use std::io::Error as IoError;
use std::io::Read;
use std::path::PathBuf;
use tiny_keccak;
use transform;

#[derive(Clone, Debug)]
pub struct Account {
    alias: String,
    private_key: String,
    public_key: String,
    address: String,
}

#[derive(Debug)]
pub enum Error {
    IoError(IoError),
    ParseError(String),
    DirNotFound,
    AliasNotFound,
    SigningError,
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IoError(ref ie) => ie.description(),
            Error::ParseError(ref msg) => msg,
            Error::DirNotFound => "Couldn't find key directory",
            Error::AliasNotFound => "Alias not found in key directory",
            Error::SigningError => "Signing failed",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        None
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Error::IoError(ref ie) => ie.fmt(f),
            Error::ParseError(ref msg) => write!(f, "ParseError: {}", msg),
            Error::DirNotFound => write!(f, "DirNotFound"),
            Error::AliasNotFound => write!(f, "AliasNotFound"),
            Error::SigningError => write!(f, "SigningError"),
        }
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::IoError(e)
    }
}

impl From<SigningError> for Error {
    fn from(e: SigningError) -> Self {
        match e {
            SigningError::ParseError(msg) => Error::ParseError(msg),
            _ => Error::ParseError(String::from("Loading pem returned a non-parse error")),
        }
    }
}

pub fn get_key_dir() -> Option<PathBuf> {
    let home = home_dir()?;

    Some([home.to_str()?, ".sawtooth", "keys"].iter().collect())
}

impl Account {
    pub fn load_from_file(key_name: &str, password: &Option<String>) -> Result<Account, Error> {
        let mut key_path = get_key_dir().ok_or(Error::DirNotFound)?;
        key_path.push(key_name);
        let pem = key_path.with_extension("pem");

        if pem.as_path().is_file() {
            Self::load_from_str(&Self::read_file(&pem)?, password)
        } else {
            Err(Error::AliasNotFound)
        }
    }

    pub fn load_from_str(key: &str, password: &Option<String>) -> Result<Account, Error> {
        let key = match (key.contains("ENCRYPTED"), password) {
            (true, Some(pw)) => Secp256k1PrivateKey::from_pem_with_password(&key.trim(), &pw),
            (true, None) => {
                return Err(Error::ParseError(
                    "A password is required for encrypted keys!".into(),
                ))
            }
            (false, Some(_)) => {
                warn!("Account::load_from_str got password for non-encrypted private key.");
                Secp256k1PrivateKey::from_pem(&key.trim())
            }
            (false, None) => Secp256k1PrivateKey::from_pem(&key.trim()),
        }?;

        let algorithm = create_context("secp256k1").unwrap();
        let pub_key = algorithm.get_public_key(&key)?;

        Ok(Account {
            alias: pub_key.as_hex(),
            private_key: key.as_hex(),
            public_key: pub_key.as_hex(),
            address: public_key_to_address(pub_key.as_slice()),
        })
    }

    fn read_file(keyfile: &PathBuf) -> Result<String, Error> {
        let mut file = File::open(keyfile.as_path().to_str().unwrap())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    pub fn sign(&self, message: &[u8]) -> Result<String, Error> {
        let algorithm = create_context("secp256k1").unwrap();
        let key =
            Secp256k1PrivateKey::from_hex(&self.private_key).map_err(|_| Error::SigningError)?;
        algorithm
            .sign(message, &key)
            .map_err(|_| Error::SigningError)
    }

    pub fn alias(&self) -> &str {
        &self.alias
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn public_key(&self) -> &str {
        &self.public_key
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Account) -> bool {
        self.private_key == other.private_key
    }
}

pub fn public_key_to_address(pub_key: &[u8]) -> String {
    transform::bytes_to_hex_str(&tiny_keccak::keccak256(pub_key)[..20])
}
