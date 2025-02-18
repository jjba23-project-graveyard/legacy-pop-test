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

use crate::keycloak::client::KeycloakClient;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum PopServiceHealthAction {
    Http { path: String, patience_seconds: i32 },
    Dummy { always_healthy: bool },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PopServicePostgres {
    pub enabled: Option<bool>,
    pub host_var: Option<String>,
    pub port_var: Option<String>,
    pub db_var: Option<String>,
    pub user_var: Option<String>,
    pub pass_var: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PopServiceKafka {
    pub enabled: Option<bool>,
    pub bootstrap_server_var: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PopServicePopServer {
    pub enabled: Option<bool>,
    pub url_vars: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PopServiceKeycloak {
    pub enabled: Option<bool>,
    pub url_var: Option<String>,
    pub realm_var: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PopService {
    pub id: String,
    pub name: Option<String>,
    pub dir: String,
    pub base_url: String,
    pub shell_path: Option<String>,
    pub start_cmd_args: Vec<String>,
    pub env_vars: Option<HashMap<String, String>>,
    pub health_check: PopServiceHealthAction,
    pub postgres: Option<PopServicePostgres>,
    pub kafka: Option<PopServiceKafka>,
    pub pop_server: Option<PopServicePopServer>,
    pub keycloak: Option<PopServiceKeycloak>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct BodyPattern {
    pub pattern: String,
    pub condition: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct HttpAssertion {
    pub name: Option<String>,
    pub delay_seconds: Option<u64>,
    pub assert_code: Option<u16>,
    pub assert_url: String,
    pub service_id: String,
    pub json_patterns: Option<Vec<BodyPattern>>,
    pub html_patterns: Option<Vec<String>>,
    pub with_keycloak_user_token: Option<bool>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct HttpCall {
    pub name: Option<String>,
    pub delay_seconds: Option<u64>,
    pub action_url: String,
    pub service_id: String,
    pub with_keycloak_user_token: Option<bool>,
    pub method: String,
    pub message_file_path: Option<String>,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct KafkaProduce {
    pub name: Option<String>,
    pub delay_seconds: Option<u64>,
    pub kafka_topic: String,
    pub message_file_path: Option<String>,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct KafkaCheck {
    pub name: Option<String>,
    pub delay_seconds: Option<u64>,
    pub kafka_topic: String,
    pub check: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PostgresAssertion {
    pub name: Option<String>,
    pub delay_seconds: Option<u64>,
    pub postgres_query: String,
    pub assert_row_count: Option<i32>,
    pub assert_expressions: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum PopAction {
    HttpAssertion(HttpAssertion),
    HttpCall(HttpCall),
    KafkaProduce(KafkaProduce),
    KafkaCheck(KafkaCheck),
    PostgresAssertion(PostgresAssertion),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PopPostgres {
    pub enabled: Option<bool>,
    pub image_name: Option<String>,
    pub image_tag: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PopKafka {
    pub enabled: Option<bool>,
    pub image_name: Option<String>,
    pub image_tag: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PopKeycloak {
    pub enabled: Option<bool>,
    pub image_name: Option<String>,
    pub image_tag: Option<String>,
    pub realm: String,
    pub client_id: String,
    pub client_secret: String,
    pub email: String,
    pub username: String,
    pub password: String,
    pub roles: Option<Vec<String>>,
    pub scopes: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PopServerBehavior {
    pub path: String,
    pub method: String,
    pub status: u16,
    pub content_type: String,
    pub body_file: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PopServer {
    pub enabled: Option<bool>,
    pub image_name: Option<String>,
    pub image_tag: Option<String>,
    pub behaviors: Option<Vec<PopServerBehavior>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PopScenario {
    pub pop_services: Option<Vec<PopService>>,
    pub pop_actions: Option<Vec<PopAction>>,
    pub pop_postgres: Option<PopPostgres>,
    pub pop_kafka: Option<PopKafka>,
    pub pop_server: Option<PopServer>,
    pub pop_keycloak: Option<PopKeycloak>,
}

#[derive(Debug, Clone)]
pub struct PopEnv {
    pub kafka_bootstrap_server: Option<String>,
    pub keycloak_client: Option<KeycloakClient>,
    pub postgres_conn_string: Option<String>,
}
