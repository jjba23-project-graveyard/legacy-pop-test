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

use std::time::Duration;

use crate::pop_model::{PopService, PopServiceHealthCheck};
use paris::{error, info, success};
use tokio::time::sleep;

pub async fn health_check(
    id: &String,
    base_url: &String,
    health_check: &PopServiceHealthCheck,
) -> Result<(), String> {
    info!("starting healthcheck for service: {}", id);
    match health_check {
        crate::pop_model::PopServiceHealthCheck::Http {
            path,
            patience_seconds,
        } => {
            let mut elapsed = 0;
            let url = format!("{}{}", base_url, path);
            while elapsed <= *patience_seconds {
                info!("calling health check URL ({}): {}", elapsed, url);
                let resp = reqwest::get(url.clone()).await;
                println!("{resp:#?}");
                match resp {
                    Err(e) => {
                        error!("{}", e);
                        sleep(Duration::from_secs(2)).await;
                        elapsed += 5;
                        continue;
                    }
                    Ok(v) => {
                        success!("{:?}", v);
                        break;
                    }
                }
            }

            Ok(())
        }
        crate::pop_model::PopServiceHealthCheck::Dummy { always_healthy } => {
            if always_healthy.to_owned() {
                Ok(())
            } else {
                Err("".to_string())
            }
        }
    }
}

pub async fn start_h(pop_service: PopService) {
    let _ = health_check(
        &pop_service.id,
        &pop_service.base_url,
        &pop_service.health_check,
    )
    .await;
}
