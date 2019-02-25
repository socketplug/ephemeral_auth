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

use serde_derive::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};

pub(crate) mod authorization;

pub(crate) fn index() -> impl Filter<Extract = (impl Reply), Error = Rejection> + Copy {
    #[derive(Deserialize, Serialize)]
    struct Return {
        status: &'static str,
    }

    warp::get2()
        .and(warp::path::end())
        .map(|| warp::reply::json(&Return { status: "ok" }))
}
