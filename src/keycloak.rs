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
use std::time::Duration;

use crate::pop_model::{PopKeycloak, PopScenarioFile, PopService};
use paris::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use testcontainers::core::ContainerPort;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CreateClientRequest {
    pub protocol: String,
    pub clientId: String,
    pub enabled: bool,
    pub publicClient: bool,
    pub secret: String,
    pub standardFlowEnabled: bool,
    pub serviceAccountsEnabled: bool,
    pub directAccessGrantsEnabled: bool,
    pub attributes: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CreateRealmRequest {
    pub enabled: bool,
    pub realm: String,
}

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
    .with_exposed_port(ContainerPort::Tcp(9000))
    .with_env_var("KC_BOOTSTRAP_ADMIN_USERNAME", "admin")
    .with_env_var("KC_BOOTSTRAP_ADMIN_PASSWORD", "admin")
    .with_env_var("DEBUG", "true")
    .with_cmd([
        "--verbose",
        "start-dev",
        "--spi-theme-static-max-age=-1",
        "--spi-theme-cache-themes=false",
        "--spi-theme-cache-templates=false",
        //        "--import-realm",
    ])
    // .with_copy_to(
    //     "/opt/keycloak/data/import/my-realm.json",
    //     PathBuf::from(
    //         "/home/joe/Ontwikkeling/Persoonlijk/pop-test/resources/realms/vandebron.json",
    //     ),
    // )
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
        .and_then(|k| k.enabled)
        .unwrap_or(false)
        && pop_service
            .keycloak
            .as_ref()
            .and_then(|k| k.enabled)
            .unwrap_or(false)
    {
        if let Some(k) = keycloak_container {
            let host = k.get_host().await.unwrap().to_string();

            let http_port = k
                .ports()
                .await
                .unwrap()
                .map_to_host_port_ipv4(8080)
                .unwrap_or(8080);

            let admin_port = k
                .ports()
                .await
                .unwrap()
                .map_to_host_port_ipv4(9000)
                .unwrap_or(9000);

            let keycloak_url = format!("http://{}:{}", host, http_port);
            let keycloak_admin_url = format!("http://{}:{}", host, admin_port);
            let realm = "vandebron".to_string();

            // todo health check on keycloak container
            tokio::time::sleep(Duration::from_secs(10)).await;

            let params = HashMap::from([
                ("username", "admin"),
                ("password", "admin"),
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
            ]);

            let get_admin_token_response = reqwest::Client::new()
                .post(format!(
                    "{}/realms/{}/protocol/openid-connect/token",
                    keycloak_url, "master"
                ))
                .header("content-type", "application/x-www-form-urlencoded")
                .form(&params)
                .send()
                .await
                .unwrap()
                .json::<Value>()
                .await
                .unwrap();

            let tok = get_admin_token_response
                .get("access_token")
                .map(|v| v.to_string())
                .unwrap_or("".to_string());
            info!("{}", tok);

            let create_realm_request = CreateRealmRequest {
                enabled: true,
                realm: realm.clone(),
            };

            let create_realm_response = reqwest::Client::new()
                .post(format!("{}/admin/realms", keycloak_admin_url))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", tok))
                .json(&create_realm_request)
                .send()
                .await;
            info!("{:?}", create_realm_response);

            let byt = create_realm_response
                .unwrap()
                .bytes()
                .await
                .unwrap()
                .to_vec();
            let bod = String::from_utf8_lossy(&byt).to_string();
            info!("{}", bod);

            let create_client_request = CreateClientRequest {
                protocol: "openid-connect".to_string(),
                clientId: "batterypack-services".to_string(),
                enabled: true,
                secret: "test".to_string(),
                publicClient: false,
                standardFlowEnabled: true,
                serviceAccountsEnabled: true,
                directAccessGrantsEnabled: true,
                attributes: HashMap::new(),
                // attributes: HashMap::from([(
                //     "oauth2.device.authorization.grant.enabled".to_string(),
                //     "true".to_string(),
                // )]),
            };

            let create_client_response = reqwest::Client::new()
                .post(format!(
                    "{}/admin/realms/{}/clients",
                    keycloak_admin_url,
                    realm.clone()
                ))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", tok))
                .json(&create_client_request)
                .send()
                .await;
            info!("{:?}", create_client_response);

            let byt = create_client_response
                .unwrap()
                .bytes()
                .await
                .unwrap()
                .to_vec();
            let bod = String::from_utf8_lossy(&byt).to_string();
            info!("{}", bod);

            let k: String = pop_service
                .keycloak
                .as_ref()
                .unwrap()
                .url_var
                .clone()
                .unwrap_or("KEYCLOAK_AUTH_SERVER_URL".to_string());
            with_env.insert(k, keycloak_url.clone());
            let k: String = pop_service
                .keycloak
                .as_ref()
                .unwrap()
                .realm_var
                .clone()
                .unwrap_or("KEYCLOAK_REALM".to_string());
            with_env.insert(k, realm);
        }
    }
}
