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

use crate::pop_model::{PopKeycloak, PopScenarioFile, PopService};
use paris::info;
use testcontainers::core::ContainerPort;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage};

pub async fn run_keycloak_container(
    pop_keycloak: PopKeycloak,
) -> Result<ContainerAsync<GenericImage>, String> {
    GenericImage::new(
        pop_keycloak
            .image_name
            .unwrap_or("keycloak/keycloak".to_string()),
        pop_keycloak.image_tag.unwrap_or("26.1".to_string()),
    )
    .with_exposed_port(ContainerPort::Tcp(8080))
    .start()
    .await
    .map_err(|e| format!("{}", e))
}

pub async fn maybe_start_keycloak(
    scenario_file: &PopScenarioFile,
) -> Result<Option<ContainerAsync<GenericImage>>, String> {
    match &scenario_file.pop_test.pop_keycloak {
        Some(keycloak) if keycloak.enabled == Some(true) => {
            info!("start setting up keycloak");
            Ok(Some(
                run_keycloak_container(keycloak.clone()).await.unwrap(),
            ))
        }
        _ => Ok(None),
    }
}

pub async fn setup_keycloak_env(
    scenario_file: &PopScenarioFile,
    pop_service: &PopService,
    keycloak_container: &Option<ContainerAsync<GenericImage>>,
    with_env: &mut HashMap<String, String>,
) {
    if scenario_file
        .pop_test
        .pop_keycloak
        .as_ref()
        .map(|k| k.enabled)
        == Some(Some(true))
        && pop_service
            .keycloak
            .as_ref()
            .map(|k| k.enabled)
            .unwrap_or(Some(false))
            == Some(true)
    {
        if let Some(k) = keycloak_container {
            let host = k.get_host().await.unwrap().to_string();
            let port = k
                .ports()
                .await
                .unwrap()
                .map_to_host_port_ipv4(8080)
                .unwrap_or(8080);
            let k: String = pop_service
                .kafka
                .as_ref()
                .unwrap()
                .bootstrap_server_var
                .clone()
                .unwrap_or("KEYCLOAK_AUTH_SERVER_URL".to_string());
            with_env.insert(k, format!("{}:{}", host, port));
        }
    }
}
