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

use std::collections::HashMap;

use crate::pop_model::{PopScenario, PopServer, PopServerBehavior, PopService};
use log::{error, info};
use reqwest::Response;
use testcontainers::core::ContainerPort;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage};
use tokio::fs;

async fn run_pop_server_container(
    pop_server: PopServer,
) -> Result<ContainerAsync<GenericImage>, String> {
    GenericImage::new(
        pop_server
            .image_name
            .unwrap_or("jjba23/pop-server".to_string()),
        pop_server.image_tag.unwrap_or("0.2.3".to_string()),
    )
    .with_exposed_port(ContainerPort::Tcp(50088))
    .start()
    .await
    .map_err(|e| format!("{}", e))
}

pub async fn maybe_start_pop_server(
    scenario: &PopScenario,
) -> Result<Option<ContainerAsync<GenericImage>>, String> {
    match &scenario.pop_server {
        Some(pop_server) if pop_server.enabled == Some(true) => {
            info!("start setting up pop_server");
            Ok(Some(
                run_pop_server_container(pop_server.clone()).await.unwrap(),
            ))
        }
        _ => Ok(None),
    }
}

pub async fn register_pop_server_behavior(
    pop_server_url: String,
    pop_server_behavior: PopServerBehavior,
) -> Result<Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let body = fs::read_to_string(pop_server_behavior.body_file)
        .await
        .unwrap_or("".to_string());

    info!("for pop_server behavior, return body: {}", body);

    client
        .post(format!(
            "{}/api/v2/pop-server/register-behavior",
            pop_server_url
        ))
        .header("x-pop-server-method", pop_server_behavior.method)
        .header("x-pop-server-path", pop_server_behavior.path)
        .header("x-pop-server-status", pop_server_behavior.status)
        .header(
            "x-pop-server-content-type",
            pop_server_behavior.content_type,
        )
        .body(body)
        .send()
        .await
}

pub async fn setup_pop_server_env(
    scenario: &PopScenario,
    pop_service: &PopService,
    pop_server_container: &Option<ContainerAsync<GenericImage>>,
    with_env: &mut HashMap<String, String>,
) {
    if scenario
        .pop_server
        .as_ref()
        .and_then(|p| p.enabled)
        .unwrap_or(false)
        && pop_service
            .pop_server
            .as_ref()
            .and_then(|p| p.enabled)
            .unwrap_or(false)
    {
        if let Some(p) = pop_server_container {
            let host = p.get_host().await.unwrap().to_string();
            let port = p
                .ports()
                .await
                .unwrap()
                .map_to_host_port_ipv4(50088)
                .unwrap_or(50088);

            for v in pop_service
                .pop_server
                .as_ref()
                .unwrap()
                .url_vars
                .as_ref()
                .unwrap()
            {
                with_env.insert(v.clone(), format!("http://{}:{}", host, port));
            }

            if let Some(bs) = scenario
                .pop_server
                .as_ref()
                .and_then(|p| p.behaviors.clone())
            {
                info!("registering behavior in pop-server: {:?}", bs);
                for b in bs {
                    match register_pop_server_behavior(format!("http://{}:{}", host, port), b).await
                    {
                        Ok(r) => info!("done registering behavior in pop-server: {:?}", r),
                        Err(e) => error!("error registering behavior in pop-server: {:?}", e),
                    }
                }
            }
        }
    }
}
