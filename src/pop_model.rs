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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum PopServiceHealthCheck {
    Http { path: String, patience_seconds: i32 },
    Dummy { always_healthy: bool },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PopService {
    pub id: String,
    pub name: String,
    pub dir: String,
    pub base_url: String,
    pub bash_path: String,
    pub start_cmd_args: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub health_check: PopServiceHealthCheck,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct BodyPattern {
    pub pattern: String,
    pub condition: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum PopCheck {
    Http {
        expected_code: i32,
        check_url: String,
        service_id: String,
        body_patterns: Vec<BodyPattern>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PopScenario {
    pub pop_services: Vec<PopService>,
    pub pop_checks: Vec<PopCheck>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PopScenarioFile {
    pub pop_test: PopScenario,
}
