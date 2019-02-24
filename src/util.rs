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

use crate::config::Config;
use http::header::HeaderValue;
use http::HeaderMap;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use reqwest::header::{COOKIE, SET_COOKIE};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct PlugResponse<T> {
    pub data: Vec<T>,
    meta: serde_json::Value,
    status: String,
    time: f64,
}

pub(crate) fn random_string(length: usize) -> String {
    let mut rng = thread_rng();
    std::iter::repeat(())
        .map(|_| rng.sample(Alphanumeric))
        .take(length)
        .collect()
}

pub(crate) fn login(config: &Config) -> Result<HeaderMap, reqwest::Error> {
    #[derive(Debug, Deserialize)]
    struct InitData {
        c: String,
        f: String,
        s: String,
        t: String,
    }

    #[derive(Debug, Serialize)]
    struct LoginPayload {
        csrf: String,
        email: String,
        password: String,
    }

    let client = reqwest::Client::new();
    let mut res = client.get("https://plug.dj/_/mobile/init").send()?;

    let mut cookies = HeaderMap::new();
    res.headers()
        .get_all(SET_COOKIE)
        .into_iter()
        .for_each(|cookie: &HeaderValue| {
            if cookie.to_str().unwrap().starts_with("session=") {
                cookies.insert(COOKIE, cookie.clone());
            }
        });

    let csrf = res
        .json::<PlugResponse<InitData>>()?
        .data
        .pop()
        .map(|i| i.c)
        .expect("No csrf returned by plug.dj init");

    let payload = LoginPayload {
        csrf,
        email: config.email.to_string(),
        password: config.password.to_string(),
    };

    let _res = client
        .post("https://plug.dj/_/auth/login")
        .headers(cookies.clone())
        .json(&payload)
        .send()?;

    // bad "status": csrfTokenInvalid

    Ok(cookies)
}
