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

use super::model::{
    CreateClientRequest, CreateRealmRequest, CreateUserRequest, CreateUserRequestCredentials,
    LoginResponse,
};
use crate::pop_model::{PopKeycloak, PopScenarioFile, PopService};
use paris::info;

use std::collections::HashMap;

use testcontainers::{ContainerAsync, GenericImage, ImageExt};

use testcontainers::core::{ContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;

pub async fn run_keycloak_container(
    pop_keycloak: PopKeycloak,
) -> Result<ContainerAsync<GenericImage>, String> {
    GenericImage::new(
        pop_keycloak
            .image_name
            .unwrap_or("keycloak/keycloak".to_string()),
        pop_keycloak.image_tag.unwrap_or("26.1".to_string()),
    )
    .with_wait_for(WaitFor::message_on_stdout(
        "Listening on: http://0.0.0.0:8080",
    ))
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
            let keycloak_container = run_keycloak_container(keycloak.clone()).await.unwrap();

            //
            //
            let host = keycloak_container.get_host().await.unwrap().to_string();

            let http_port = keycloak_container
                .ports()
                .await
                .unwrap()
                .map_to_host_port_ipv4(8080)
                .unwrap_or(8080);

            let keycloak_url = format!("http://{}:{}", host, http_port);

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
                .json::<LoginResponse>()
                .await
                .unwrap();

            let tok = get_admin_token_response.access_token;
            info!("admin token: {}", tok);

            //
            //

            let realm = keycloak.realm.clone();

            let create_realm_request = CreateRealmRequest {
                enabled: true,
                realm: realm.clone(),
                verifyEmail: false,
            };

            let create_realm_response = reqwest::Client::new()
                .post(format!("{}/admin/realms", keycloak_url))
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
            info!("create_realm_response: {}", bod);

            //
            //

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
                    keycloak_url,
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
            info!("create_client_response: {}", bod);

            //
            //

            let create_user_request = CreateUserRequest {
                username: "batterypack-services".to_string(),
                enabled: true,
                firstName: "battery".to_string(),
                lastName: "pack".to_string(),
                email: "battery@pack.com".to_string(),
                emailVerified: true,
                credentials: vec![CreateUserRequestCredentials {
                    r#type: "password".to_string(),
                    value: "password".to_string(),
                    temporary: false,
                }],
            };

            let create_user_response = reqwest::Client::new()
                .post(format!(
                    "{}/admin/realms/{}/users",
                    keycloak_url,
                    realm.clone()
                ))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", tok))
                .json(&create_user_request)
                .send()
                .await;
            info!("{:?}", create_user_response);

            let byt = create_user_response
                .unwrap()
                .bytes()
                .await
                .unwrap()
                .to_vec();
            let bod = String::from_utf8_lossy(&byt).to_string();
            info!("create_user_response: {}", bod);

            //
            //

            let params = HashMap::from([
                ("username", "batterypack-services"),
                ("password", "password"),
                ("grant_type", "password"),
                ("client_id", "batterypack-services"),
                ("client_secret", "test"),
            ]);

            let get_user_token_response = reqwest::Client::new()
                .post(format!(
                    "{}/realms/{}/protocol/openid-connect/token",
                    keycloak_url, realm
                ))
                .header("content-type", "application/x-www-form-urlencoded")
                .form(&params)
                .send()
                .await;

            let byt = get_user_token_response
                .unwrap()
                .bytes()
                .await
                .unwrap()
                .to_vec();
            let bod = String::from_utf8_lossy(&byt).to_string();
            info!("get_user_token_response: {}", bod);
            // let tok = get_user_token_response.access_token;
            // info!("get_user_token_response: {}", tok);

            //
            //

            Ok(Some(keycloak_container))
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
        .clone()
        .and_then(|k| k.enabled)
        == Some(true)
        && pop_service.keycloak.clone().and_then(|k| k.enabled) == Some(true)
    {
        if let Some(container) = keycloak_container {
            let host = container.get_host().await.unwrap();
            let http_port = container
                .ports()
                .await
                .unwrap()
                .map_to_host_port_ipv4(8080)
                .unwrap_or(8080);
            let keycloak_url = format!("http://{}:{}", host, http_port);

            let keycloak = pop_service.keycloak.as_ref().unwrap();

            with_env.insert(
                keycloak
                    .url_var
                    .clone()
                    .unwrap_or_else(|| "KEYCLOAK_AUTH_SERVER_URL".to_string()),
                keycloak_url,
            );
            with_env.insert(
                keycloak
                    .realm_var
                    .clone()
                    .unwrap_or_else(|| "KEYCLOAK_REALM".to_string()),
                scenario_file
                    .pop_test
                    .pop_keycloak
                    .as_ref()
                    .unwrap()
                    .realm
                    .clone(),
            );
        }
    }
}
