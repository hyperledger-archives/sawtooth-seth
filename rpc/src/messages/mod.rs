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

pub mod seth;

use messages::seth::EvmPermissions;
use std::fmt;
use std::str::FromStr;

impl FromStr for EvmPermissions {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut permissions = EvmPermissions::new();

        for perm in s.split(',') {
            let mut chars = perm.chars();
            let modifier = chars
                .next()
                .ok_or_else(|| String::from("Found an empty string instead of a permission!"))?;
            let name = chars.collect::<String>();

            let perm_bit = match name.as_str() {
                "all" => 1 | 2 | 4 | 8 | 16,
                "root" => 1,
                "send" => 2,
                "call" => 4,
                "contract" => 8,
                "account" => 16,
                _ => return Err(format!("Unknown permission `{}`", name)),
            };

            match modifier {
                '+' => permissions.perms |= perm_bit,
                '-' => permissions.perms &= !perm_bit,
                _ => return Err(format!("Bad modifier `{}`", modifier)),
            }
        }

        Ok(permissions)
    }
}

impl fmt::Display for EvmPermissions {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let perms = [
            ("root", 1),
            ("send", 2),
            ("call", 4),
            ("contract", 8),
            ("account", 16),
        ];

        let perms_str = perms
            .iter()
            .map(|(name, bit)| format!("{}{}", if self.perms & bit == 0 { "-" } else { "+" }, name))
            .collect::<Vec<_>>()
            .join(",");

        fmt.write_str(&perms_str)?;
        Ok(())
    }
}
