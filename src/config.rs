/*
 * sepha: A self contained 3rd party auth server for plug.dj
 * Copyright (C) 2019 Chip Reed
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use serde_derive::Deserialize;
use std::fs;
use std::io::Error;
use std::path::PathBuf;

const DEFAULT_CONFIG_PATH: &str = ".sepha.json";

#[derive(Deserialize)]
pub(crate) struct Config {
    pub email: String,
    pub password: String,
}

impl Config {
    pub fn load(path: Option<PathBuf>) -> Result<Config, Error> {
        let path = path.unwrap_or(DEFAULT_CONFIG_PATH.into());
        let content = fs::read_to_string(&path)?;
        let config: Config = serde_json::from_str(&content).expect("Unable to parse config file.");
        Ok(config)
    }
}
