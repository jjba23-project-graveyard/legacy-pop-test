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

use super::model::CreateRealmRequest;
use crate::keycloak::model::CreateClientRequest;
use paris::info;
use reqwest::Client;
use std::collections::HashMap;

pub struct KeycloakAdminToken(pub String);
pub struct KeycloakRealm(pub String);
pub struct KeycloakUrl(pub String);
pub struct KeycloakClientSecret(pub String);
pub struct KeycloakClientId(pub String);
pub struct KeycloakUsername(pub String);
pub struct KeycloakPassword(pub String);

async fn send_keycloak_request(
    url: String,
    admin_token: &KeycloakAdminToken,
    body: impl serde::Serialize,
) -> Result<(), String> {
    let response = Client::new()
        .post(url)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", admin_token.0))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let body_bytes = response.bytes().await.map_err(|e| e.to_string())?.to_vec();
    let body_str = String::from_utf8_lossy(&body_bytes).to_string();

    info!("Response: {}", body_str);
    Ok(())
}

pub async fn keycloak_create_realm(
    admin_token: KeycloakAdminToken,
    realm: KeycloakRealm,
    url: KeycloakUrl,
) -> Result<(), String> {
    let request_body = CreateRealmRequest {
        enabled: true,
        realm: realm.0,
        verify_email: false,
    };

    send_keycloak_request(
        format!("{}/admin/realms", url.0),
        &admin_token,
        request_body,
    )
    .await
}

pub async fn keycloak_create_client(
    admin_token: KeycloakAdminToken,
    realm: KeycloakRealm,
    url: KeycloakUrl,
    client_id: KeycloakClientId,
    client_secret: KeycloakClientSecret,
) -> Result<(), String> {
    let request_body = CreateClientRequest {
        protocol: "openid-connect".to_string(),
        client_id: client_id.0,
        enabled: true,
        secret: client_secret.0,
        public_client: false,
        standard_flow_enabled: true,
        service_accounts_enabled: true,
        direct_access_grants_enabled: true,
        attributes: HashMap::new(),
    };

    send_keycloak_request(
        format!("{}/admin/realms/{}/clients", url.0, realm.0),
        &admin_token,
        request_body,
    )
    .await
}

pub async fn keycloak_get_user_token(
    realm: KeycloakRealm,
    url: KeycloakUrl,
    client_id: KeycloakClientId,
    client_secret: KeycloakClientSecret,
    username: KeycloakUsername,
    password: KeycloakPassword,
) -> Result<(), String> {
    let params = HashMap::from([
        ("username", username.0.as_str()),
        ("password", password.0.as_str()),
        ("grant_type", "password"),
        ("client_id", client_id.0.as_str()),
        ("client_secret", client_secret.0.as_str()),
    ]);

    let get_user_token_response = reqwest::Client::new()
        .post(format!(
            "{}/realms/{}/protocol/openid-connect/token",
            url.0, realm.0
        ))
        .header("content-type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let byt = get_user_token_response
        .bytes()
        .await
        .map_err(|e| e.to_string())?
        .to_vec();

    let bod = String::from_utf8_lossy(&byt).to_string();

    info!("get_user_token_response: {}", bod);

    Ok(())
}
