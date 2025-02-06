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

use crate::pop_model::{BodyPattern, PopCheck, PopScenarioFile};

use crate::condition_dsl::eval_condition;
use paris::{error, info, success};

pub async fn perform_checks(scenario_file: &PopScenarioFile) {
    let services = &scenario_file.pop_test.pop_services;

    for check in &scenario_file
        .pop_test
        .pop_checks
        .clone()
        .unwrap_or_default()
    {
        match check {
            PopCheck::Http {
                expected_code,
                check_url,
                service_id,
                json_body_patterns,
            } => {
                if let Some(service) = services.iter().find(|s| s.id == *service_id) {
                    info!("found service to check: {:?}", service);
                    info!("running check: {:?}", check);
                    if let Err(e) = do_with_http_check(
                        *expected_code,
                        check_url.clone(),
                        service.base_url.clone(),
                        json_body_patterns.clone(),
                    )
                    .await
                    {
                        error!("HTTP check failed: {:?}", e);
                    }
                } else {
                    error!("could not find service details for check");
                }
            }
        }
    }
}

pub async fn do_with_http_check(
    expected_code: Option<i32>,
    check_url: String,
    service_url: String,
    json_body_patterns: Option<Vec<BodyPattern>>,
) -> Result<(), String> {
    let url = format!("{}{}", service_url, check_url);

    let maybe_response = reqwest::get(url.clone()).await;
    println!("{maybe_response:#?}");
    let wanted_status_code = expected_code.unwrap_or(200);

    match maybe_response {
        Err(e) => error!("{}", e),
        Ok(resp) => {
            if (resp.status().as_u16() as i32) != wanted_status_code {
                return Err("the HTTP status did not match the expectation".to_string());
            } else {
                success!(
                    "the HTTP response contained the expected status: {:?}",
                    wanted_status_code
                )
            }

            let j = &resp.json().await;
            if let Ok(body) = j {
                eval_condition(body, json_body_patterns.unwrap_or_default()).unwrap()
            }
        }
    }

    Ok(())
}
