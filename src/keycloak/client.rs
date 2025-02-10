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

use crate::keycloak::model::CreateClientRequest;

use super::model::CreateRealmRequest;
use paris::info;

pub struct KeycloakAdminToken(pub String);
pub struct KeycloakRealm(pub String);
pub struct KeycloakUrl(pub String);
pub struct KeycloakClientSecret(pub String);
pub struct KeycloakClientId(pub String);

pub async fn keycloak_create_realm(
    admin_token: KeycloakAdminToken,
    realm: KeycloakRealm,
    url: KeycloakUrl,
) -> Result<(), String> {
    let create_realm_request = CreateRealmRequest {
        enabled: true,
        realm: realm.0,
        verify_email: false,
    };

    let create_realm_response = reqwest::Client::new()
        .post(format!("{}/admin/realms", url.0))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", admin_token.0))
        .json(&create_realm_request)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let byt = create_realm_response
        .bytes()
        .await
        .map_err(|e| e.to_string())?
        .to_vec();

    let bod = String::from_utf8_lossy(&byt).to_string();

    info!("create_realm_response: {}", bod);

    Ok(())
}

pub async fn keycloak_create_client(
    admin_token: KeycloakAdminToken,
    realm: KeycloakRealm,
    url: KeycloakUrl,
    client_id: KeycloakClientId,
    client_secret: KeycloakClientSecret,
) -> Result<(), String> {
    let create_client_request = CreateClientRequest {
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

    let create_client_response = reqwest::Client::new()
        .post(format!("{}/admin/realms/{}/clients", url.0, realm.0))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", admin_token.0))
        .json(&create_client_request)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let byt = create_client_response
        .bytes()
        .await
        .map_err(|e| e.to_string())?
        .to_vec();

    let bod = String::from_utf8_lossy(&byt).to_string();
    info!("create_client_response: {}", bod);

    Ok(())
}
