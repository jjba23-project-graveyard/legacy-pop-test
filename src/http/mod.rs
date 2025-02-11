// pop-test - your test orchestration master of puppets

// Copyright (C) 2025 Josep Bigorra (jjba23) <jjbigorra@gmail.com>

// pop-test is free software: you can redistribute it and/or modify
// it under the terms of the Lesser GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// pop-test is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// Lesser GNU General Public License for more details.

// You should have received a copy of the Lesser GNU General Public License
// along with pop-test.  If not, see <https://www.gnu.org/licenses/>.

use crate::pop_model::BodyPattern;

use crate::condition_dsl::eval_condition;
use log::{debug, error, info};
use reqwest::{
    header::{self, HeaderValue},
    Client,
};

pub async fn run_http_assertion(
    expected_code: Option<u16>,
    assert_url: String,
    service_url: String,
    json_body_patterns: Option<Vec<BodyPattern>>,
    bearer_token: Option<String>,
) -> Result<(), String> {
    let url = format!("{}{}", service_url, assert_url);

    let mut headers = header::HeaderMap::new();
    if let Some(tok) = bearer_token {
        headers.insert(
            "Authorization",
            HeaderValue::from_str(format!("Bearer {}", tok).as_str()).map_err(|e| e.to_string())?,
        );
    }

    let c = Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| e.to_string())?
        .get(url);

    let maybe_response = c.send().await;

    println!("{maybe_response:#?}");
    let wanted_status_code = expected_code.unwrap_or(200);

    match maybe_response {
        Err(e) => error!("{}", e),
        Ok(resp) => {
            if resp.status().as_u16() != wanted_status_code {
                error!("the HTTP status did not match the expectation")
            } else {
                info!(
                    "the HTTP response contained the expected status: {:?}",
                    wanted_status_code
                )
            }

            let body_bytes = &resp.bytes().await.map_err(|e| e.to_string())?.to_vec();
            let body_str = String::from_utf8_lossy(body_bytes).to_string();
            debug!("response body from assertion: {}", body_str);

            let j = serde_json::from_slice(body_bytes).map_err(|e| e.to_string());
            if let Ok(body) = &j {
                eval_condition(body, json_body_patterns.unwrap_or_default()).unwrap()
            }
        }
    }

    Ok(())
}
