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

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum PopServiceHealthCheck {
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
pub struct PopService {
    pub id: String,
    pub name: Option<String>,
    pub dir: String,
    pub base_url: String,
    pub shell_path: Option<String>,
    pub start_cmd_args: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub health_check: PopServiceHealthCheck,
    pub postgres: Option<PopServicePostgres>,
    pub kafka: Option<PopServiceKafka>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct BodyPattern {
    pub pattern: String,
    pub condition: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum PopCheck {
    Http {
        expected_code: Option<i32>,
        check_url: String,
        service_id: String,
        json_body_patterns: Option<Vec<BodyPattern>>,
    },
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
pub struct PopServer {
    pub enabled: Option<bool>,
    pub image_name: Option<String>,
    pub image_tag: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PopScenario {
    pub pop_services: Vec<PopService>,
    pub pop_checks: Option<Vec<PopCheck>>,
    pub pop_postgres: Option<PopPostgres>,
    pub pop_kafka: Option<PopKafka>,
    pub pop_server: Option<PopServer>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PopScenarioFile {
    pub pop_test: PopScenario,
}
