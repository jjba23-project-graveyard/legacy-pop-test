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

use crate::pop_model::{PopServer, PopServerBehavior};
use reqwest::Response;
use testcontainers::core::ContainerPort;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage};
use tokio::fs;

pub async fn start_pop_server_container(
    pop_server: PopServer,
) -> Result<ContainerAsync<GenericImage>, String> {
    GenericImage::new(
        pop_server
            .image_name
            .unwrap_or("jjba23/pop-server".to_string()),
        pop_server.image_tag.unwrap_or("0.2.2".to_string()),
    )
    .with_exposed_port(ContainerPort::Tcp(50088))
    .start()
    .await
    .map_err(|e| format!("{}", e))
}

pub async fn register_pop_server_behavior(
    pop_server_url: String,
    pop_server_behavior: PopServerBehavior,
) -> Result<Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let body = fs::read_to_string(pop_server_behavior.body_file)
        .await
        .unwrap_or("".to_string());

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
