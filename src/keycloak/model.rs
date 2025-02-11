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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateClientRequest {
    pub protocol: String,
    pub client_id: String,
    pub enabled: bool,
    pub public_client: bool,
    pub secret: String,
    pub standard_flow_enabled: bool,
    pub service_accounts_enabled: bool,
    pub direct_access_grants_enabled: bool,
    pub attributes: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleRequest {
    pub name: String,
    pub scope_param_required: bool,
    pub composite: bool,
    pub client_role: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoleRepresentation {
    pub id: String,
    pub name: String,
    pub composite: bool,
    pub client_role: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleMappingRequest {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateRealmRequest {
    pub enabled: bool,
    pub realm: String,
    pub verify_email: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginResponse {
    pub access_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateUserRequestCredentials {
    pub r#type: String,
    pub value: String,
    pub temporary: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub username: String,
    pub enabled: bool,
    pub credentials: Vec<CreateUserRequestCredentials>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub email_verified: bool,
}
