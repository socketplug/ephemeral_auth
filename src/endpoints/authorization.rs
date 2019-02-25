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

use crate::util;
use crate::util::random_string;
use crate::util::PlugResponse;
use chrono::Utc;
use jsonwebtoken::Validation;
use jsonwebtoken::{decode, encode, Header};
use reqwest::Client;
use serde_derive::{Deserialize, Serialize};
use std::borrow::Cow;
use std::error::Error;
use time::Duration;
use warp::{Filter, Rejection, Reply};

#[derive(Debug, Clone)]
pub(crate) struct AuthorizationConfig<'a> {
    pub(crate) base_path: &'a str,
    pub(crate) issuer: &'a str,
    pub(crate) private_key: &'a [u8],
}

impl<'a> Default for AuthorizationConfig<'a> {
    fn default() -> Self {
        Self {
            base_path: "auth",
            issuer: "sepha",
            private_key: &[],
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct InitClaims<'a> {
    /// registered claim: expiration time
    exp: i64,

    /// registered claim: issued at
    iat: i64,

    /// registered claim: subject
    sub: Cow<'a, str>,

    // our claims
    id: u64,
    public_token: Cow<'a, str>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AuthenticationClaims<'a> {
    /// registered claim: expiration time
    exp: i64,

    /// registered claim: issued at
    iat: i64,

    /// registered claim: subject
    sub: Cow<'a, str>,

    // our claims
    id: u64,
}

/// Endpoint to start authentication process
///
/// Url is `/<AuthorizationConfig.base_path>/init/:id` where `:id` is a u64
pub(crate) fn init(
    config: &'static AuthorizationConfig,
) -> impl Filter<Extract = (impl Reply), Error = Rejection> + Copy {
    #[derive(Debug, Deserialize, Serialize)]
    struct Return {
        public_token: String,
        secret: String,
    }

    warp::get2()
        .and(warp::path(&config.base_path))
        .and(warp::path("init"))
        .and(warp::path::param2())
        .and(warp::path::end())
        .map(move |plug_id: u64| {
            let public_token = random_string(64);

            // how long the token is valid for
            let exp = Utc::now() + Duration::minutes(5);

            let secret = encode(
                &Header::default(),
                &InitClaims {
                    exp: exp.timestamp(),
                    iat: Utc::now().timestamp(),
                    sub: Cow::Borrowed("init"),
                    id: plug_id,
                    public_token: Cow::Borrowed(&public_token),
                },
                &config.private_key,
            )
            .expect("authorization: action_create: could not create jwt");

            warp::reply::json(&Return {
                public_token,
                secret,
            })
        })
}

/// Endpoint to authenticate a prepared account
///
/// Url is `/<AuthorizationConfig.base_path>/authenticate` with the secret token
/// received from init as a JSON posted string.
///
/// Authenticated account tokens are by default valid for 1 hour
pub(crate) fn authenticate(
    config: &'static AuthorizationConfig,
    verify_client: &'static Client,
) -> impl Filter<Extract = (impl Reply), Error = Rejection> + Copy {
    #[derive(Debug, Deserialize)]
    struct BlurbData {
        blurb: String,
    }

    #[derive(Debug, Serialize)]
    struct Return<'a> {
        valid: Cow<'a, str>,
        token: Option<String>,
    }

    warp::post2()
        .and(warp::path(&config.base_path))
        .and(warp::path("authenticate"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024)) // 1kb body limit
        .and(warp::body::json())
        .map(move |raw_claims: String| {
            match decode::<InitClaims>(
                &raw_claims,
                &config.private_key,
                &Validation {
                    sub: Some("init".to_string()),
                    ..Default::default()
                },
            ) {
                Ok(claims) => {
                    let mut res = verify_client
                        .get(&format!(
                            "https://plug.dj/_/profile/{}/blurb",
                            claims.claims.id
                        ))
                        .send()
                        .expect("Could not fetch plug api to grab blurb");

                    let blurb = res
                        .json::<PlugResponse<BlurbData>>()
                        .expect("Unknown blurb response format")
                        .data
                        .pop()
                        .map(|bd| bd.blurb)
                        .expect("No csrf returned by plug.dj init");

                    if &blurb == &claims.claims.public_token {
                        let exp = Utc::now() + Duration::hours(1);
                        let valid_token = encode(
                            &Header::default(),
                            &AuthenticationClaims {
                                exp: exp.timestamp(),
                                iat: Utc::now().timestamp(),
                                sub: Cow::Borrowed("auth_token"),
                                id: claims.claims.id,
                            },
                            &config.private_key,
                        )
                        .expect("authorization: action_authenticate: could not create jwt");

                        warp::reply::json(&Return {
                            valid: Cow::Borrowed("valid"),
                            token: Some(valid_token),
                        })
                    } else {
                        warp::reply::json(&Return {
                            valid: Cow::Borrowed("invalid public_token"),
                            token: None,
                        })
                    }
                }
                Err(e) => warp::reply::json(&Return {
                    valid: Cow::Owned(e.description().to_string()),
                    token: None,
                }),
            }
        })
}

/// Endpoint to verify an authenticated account token
///
/// Url is `/<AuthorizationConfig.base_path>/verify` with the authenticated account token
/// received from authenticate as a JSON posted string.
pub(crate) fn verify(
    config: &'static AuthorizationConfig,
) -> impl Filter<Extract = (impl Reply), Error = Rejection> + Copy {
    #[derive(Debug, Serialize)]
    struct Return {
        verify: bool,
    }

    warp::post2()
        .and(warp::path(&config.base_path))
        .and(warp::path("verify"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024)) // 1kb body limit
        .and(warp::body::json())
        .map(move |raw_claims: String| {
            warp::reply::json(&Return {
                verify: decode::<AuthenticationClaims>(
                    &raw_claims,
                    &config.private_key,
                    &Validation {
                        sub: Some("auth_token".to_string()),
                        ..Default::default()
                    },
                )
                .is_ok(),
            })
        })
}
