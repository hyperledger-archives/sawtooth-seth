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

//! Initialize CLI configuration

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use client::{Client, Settings};
use dirs;
use failure::Error;
use std::fs;
use toml::to_string;

/// Returns Clap configuration
pub fn get_cli<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("config")
        .about("Manages seth config")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommands(vec![
            SubCommand::with_name("init")
                .about("Initializes seth config")
                .args(&[Arg::with_name("url")
                    .long("--url")
                    .takes_value(true)
                    .help("The URL of the JSON-RPC API")]),
        ])
}

/// Handles parsing Clap CLI matches
pub fn parse_cli<'a>(matches: (&'a str, Option<&'a ArgMatches>)) -> Result<(), Error> {
    match matches {
        ("init", Some(m)) => {
            let url = m.value_of("url");

            do_init(url)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

/// Initializes Seth CLI configuration file
pub fn do_init(url: Option<&str>) -> Result<(), Error> {
    let client = Client::new()?;

    let settings = Settings {
        url: url
            .or_else(|| Some(&client.url))
            .expect("Client URL must exist!")
            .into(),
    };

    let toml = to_string(&settings)?;

    let mut settings_file =
        dirs::home_dir().ok_or_else(|| format_err!("Couldn't find home directory!"))?;

    settings_file.push(".sawtooth");
    settings_file.push("seth-config.toml");

    fs::write(settings_file, toml)?;

    Ok(())
}
