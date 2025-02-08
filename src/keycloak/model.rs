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
pub struct CreateClientRequest {
    pub protocol: String,
    #[allow(non_snake_case)]
    pub clientId: String,
    pub enabled: bool,
    #[allow(non_snake_case)]
    pub publicClient: bool,
    pub secret: String,
    #[allow(non_snake_case)]
    pub standardFlowEnabled: bool,
    #[allow(non_snake_case)]
    pub serviceAccountsEnabled: bool,
    #[allow(non_snake_case)]
    pub directAccessGrantsEnabled: bool,
    pub attributes: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateRealmRequest {
    pub enabled: bool,
    pub realm: String,
    #[allow(non_snake_case)]
    pub verifyEmail: bool,
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
pub struct CreateUserRequest {
    pub username: String,
    pub enabled: bool,
    pub credentials: Vec<CreateUserRequestCredentials>,
    #[allow(non_snake_case)]
    pub firstName: String,
    #[allow(non_snake_case)]
    pub lastName: String,
    pub email: String,
    #[allow(non_snake_case)]
    pub emailVerified: bool,
}
