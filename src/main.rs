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
use crate::endpoints::{authorization, authorization::AuthorizationConfig};
use lazy_static::lazy_static;
use reqwest::Client;
use std::env;
use std::path::PathBuf;
use warp::Filter;

mod config;
mod endpoints;
mod util;

lazy_static! {
    static ref PRIVATE_KEY: Vec<u8> =
        std::fs::read("private.der").expect("could not read key private.der");
    static ref AUTHORIZATION_CONFIG: AuthorizationConfig<'static> = AuthorizationConfig {
        private_key: &PRIVATE_KEY,
        ..Default::default()
    };
    static ref VERIFY_CLIENT: Client = {
        let config =
            Config::load(env::args().nth(1).map(PathBuf::from)).expect("Cannot get config");
        let login = util::login(&config);
        dbg!(&login);

        Client::builder()
            .default_headers(login.unwrap())
            .build()
            .expect("couldnt build async request client")
    };
}

fn main() {
    // authorization routes
    let authorization_router = authorization::init(&AUTHORIZATION_CONFIG)
        .or(authorization::authenticate(
            &AUTHORIZATION_CONFIG,
            &VERIFY_CLIENT,
        ))
        .or(authorization::verify(&AUTHORIZATION_CONFIG));

    // all routes
    let router = authorization_router.or(endpoints::index());

    warp::serve(router).run(([0, 0, 0, 0], 8080));
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(1 + 1, 2);
    }
}
