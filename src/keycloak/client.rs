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

use super::model::{CreateRealmRequest, CreateRoleRequest};
use crate::keycloak::model::{
    CreateClientRequest, CreateUserRequest, CreateUserRequestCredentials, LoginResponse,
};
use log::debug;
use reqwest::Client;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct KeycloakAdminToken(pub String);

#[derive(Debug, Clone)]
pub struct KeycloakRealm(pub String);

#[derive(Debug, Clone)]
pub struct KeycloakUrl(pub String);

#[derive(Debug, Clone)]
pub struct KeycloakClientSecret(pub String);

#[derive(Debug, Clone)]
pub struct KeycloakClientId(pub String);

#[derive(Debug, Clone)]
pub struct KeycloakUsername(pub String);

#[derive(Debug, Clone)]
pub struct KeycloakUserToken(pub String);

#[derive(Debug, Clone)]
pub struct KeycloakPassword(pub String);

#[derive(Debug, Clone)]
pub struct KeycloakRoleName(pub String);

#[derive(Debug, Clone)]
pub struct KeycloakClient {
    pub url: KeycloakUrl,
}

impl KeycloakClient {
    pub async fn send_request(
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

        debug!("got response from keycloak: {:?}", response);

        let body_bytes = response.bytes().await.map_err(|e| e.to_string())?.to_vec();
        let body_str = String::from_utf8_lossy(&body_bytes).to_string();

        debug!("response body from keycloak: {}", body_str);
        Ok(())
    }

    pub async fn create_realm(
        &mut self,
        admin_token: KeycloakAdminToken,
        realm: KeycloakRealm,
    ) -> Result<(), String> {
        let request_body = CreateRealmRequest {
            enabled: true,
            realm: realm.0,
            verify_email: false,
        };

        Self::send_request(
            format!("{}/admin/realms", self.url.0),
            &admin_token,
            request_body,
        )
        .await
    }

    pub async fn create_client(
        &mut self,
        admin_token: KeycloakAdminToken,
        realm: KeycloakRealm,
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

        Self::send_request(
            format!("{}/admin/realms/{}/clients", self.url.0, realm.0),
            &admin_token,
            request_body,
        )
        .await
    }

    pub async fn admin_token(&mut self) -> Result<KeycloakAdminToken, String> {
        let params = HashMap::from([
            ("username", "admin"),
            ("password", "admin"),
            ("grant_type", "password"),
            ("client_id", "admin-cli"),
        ]);

        let master_realm = "master";
        let response = reqwest::Client::new()
            .post(format!(
                "{}/realms/{}/protocol/openid-connect/token",
                self.url.0, master_realm
            ))
            .header("content-type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<LoginResponse>()
            .await
            .map_err(|e| e.to_string())?;

        debug!("got response from keycloak: {:?}", response);

        Ok(KeycloakAdminToken(response.access_token))
    }

    pub async fn create_user(
        &mut self,
        admin_token: KeycloakAdminToken,
        realm: KeycloakRealm,
        username: KeycloakUsername,
        password: KeycloakPassword,
    ) -> Result<(), String> {
        let create_user_request = CreateUserRequest {
            username: username.0,
            enabled: true,
            first_name: "test".to_string(),
            last_name: "test".to_string(),
            email: "test@test.com".to_string(),
            email_verified: true,
            credentials: vec![CreateUserRequestCredentials {
                r#type: "password".to_string(),
                value: password.0,
                temporary: false,
            }],
        };

        Self::send_request(
            format!("{}/admin/realms/{}/users", self.url.0, realm.0),
            &admin_token,
            &create_user_request,
        )
        .await
    }

    pub async fn get_user_token(
        &mut self,
        realm: KeycloakRealm,
        client_id: KeycloakClientId,
        client_secret: KeycloakClientSecret,
        username: KeycloakUsername,
        password: KeycloakPassword,
    ) -> Result<KeycloakUserToken, String> {
        let params = HashMap::from([
            ("username", username.0.as_str()),
            ("password", password.0.as_str()),
            ("grant_type", "password"),
            ("client_id", client_id.0.as_str()),
            ("client_secret", client_secret.0.as_str()),
        ]);

        let response = reqwest::Client::new()
            .post(format!(
                "{}/realms/{}/protocol/openid-connect/token",
                self.url.0, realm.0
            ))
            .header("content-type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<LoginResponse>()
            .await
            .map_err(|e| e.to_string())?;

        debug!("got response from keycloak: {:?}", response);

        Ok(KeycloakUserToken(response.access_token))
    }

    pub async fn create_role(
        &mut self,
        admin_token: KeycloakAdminToken,
        realm: KeycloakRealm,
        role_name: KeycloakRoleName,
    ) -> Result<(), String> {
        let request_body = CreateRoleRequest {
            name: role_name.0,
            scope_param_required: false,
            composite: false,
        };

        Self::send_request(
            format!("{}/admin/realms/{}/roles", self.url.0, realm.0),
            &admin_token,
            request_body,
        )
        .await
    }
}
