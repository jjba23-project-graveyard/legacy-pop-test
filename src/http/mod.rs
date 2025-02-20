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

use std::{collections::HashMap, str::FromStr};

use crate::pop_model::BodyPattern;
use scraper::{ElementRef, Html, Selector};
use serde_json::Value;

use crate::json_condition_dsl::eval_json_condition;
use log::{debug, error, info};
use reqwest::{
    header::{self, HeaderName, HeaderValue},
    Client, Method,
};

pub async fn run_http_assertion(
    expected_code: Option<u16>,
    assert_url: String,
    service_url: String,
    json_patterns: Option<Vec<BodyPattern>>,
    html_patterns: Option<Vec<String>>,
    bearer_token: Option<String>,
    method: String,
    extra_headers: &HashMap<String, String>,
    msg: String,
) -> Result<(), String> {
    let url = format!("{}{}", service_url, assert_url);

    let mut headers = header::HeaderMap::new();
    if let Some(tok) = bearer_token {
        headers.insert(
            "Authorization",
            HeaderValue::from_str(format!("Bearer {}", tok).as_str()).map_err(|e| e.to_string())?,
        );
    }

    for (k, v) in extra_headers.clone() {
        headers.insert(
            HeaderName::from_str(&k).map_err(|e| e.to_string())?,
            HeaderValue::from_str(&v).map_err(|e| e.to_string())?,
        );
    }

    let c = Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| e.to_string())?
        .request(
            Method::from_str(method.as_str()).unwrap_or(Method::GET),
            url,
        )
        .body(msg);

    let maybe_response = c.send().await;

    debug!("{maybe_response:#?}");
    let wanted_status_code = expected_code.unwrap_or(200);

    match maybe_response {
        Err(e) => Err(e.to_string()),
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

            debug!("\n{}", body_str);

            match (json_patterns, html_patterns) {
                (None, Some(the_html_patterns)) => {
                    let document: Html = Html::parse_document(body_str.as_str());
                    for p in the_html_patterns {
                        let selector = Selector::parse(&p).unwrap();
                        let m: Vec<ElementRef> = document.select(&selector).collect();
                        info!("found {} elements that match pattern", m.len());
                    }

                    Ok(())
                }
                (Some(the_json_patterns), None) => {
                    let body: Value =
                        serde_json::from_slice(body_bytes).map_err(|e| e.to_string())?;
                    let res = the_json_patterns
                        .iter()
                        .map(|pattern| eval_json_condition(&body, pattern.clone()));
                    let collected: Result<(), String> = res.into_iter().collect();
                    collected
                }
                (Some(_), Some(_)) => {
                    Err("cannot perform JSON and HTML patterns simultaneously ".to_string())
                }
                (None, None) => Ok(()),
            }
        }
    }
}

pub async fn run_http_action(
    assert_url: String,
    service_url: String,
    bearer_token: Option<String>,
    method: String,
    extra_headers: &HashMap<String, String>,
    msg: String,
) -> Result<(), String> {
    let url = format!("{}{}", service_url, assert_url);

    let mut headers = header::HeaderMap::new();
    if let Some(tok) = bearer_token {
        headers.insert(
            "Authorization",
            HeaderValue::from_str(format!("Bearer {}", tok).as_str()).map_err(|e| e.to_string())?,
        );
    }

    for (k, v) in extra_headers.clone() {
        headers.insert(
            HeaderName::from_str(&k).map_err(|e| e.to_string())?,
            HeaderValue::from_str(&v).map_err(|e| e.to_string())?,
        );
    }

    info!("sending request body: {}", msg);

    let c = Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| e.to_string())?
        .request(
            Method::from_str(method.as_str()).unwrap_or(Method::POST),
            url,
        )
        .body(msg);

    let maybe_response = c.send().await;

    info!("{maybe_response:#?}");

    match maybe_response {
        Err(e) => error!("{}", e),
        Ok(resp) => {
            let body_bytes = &resp.bytes().await.map_err(|e| e.to_string())?.to_vec();
            let body_str = String::from_utf8_lossy(body_bytes).to_string();
            debug!("response body from assertion: {}", body_str);
        }
    }

    Ok(())
}
