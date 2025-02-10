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

use crate::kafka::container::setup_kafka_env;
use crate::keycloak::container::setup_keycloak_env;
use crate::pop_model::{PopScenarioFile, PopService};
use crate::pop_server::setup_pop_server_env;
use crate::postgres::container::setup_postgres_env;
use std::collections::HashMap;
use testcontainers::{ContainerAsync, GenericImage};
use testcontainers_modules::kafka::Kafka;
use testcontainers_modules::postgres::Postgres;

pub async fn setup_environment(
    scenario_file: &PopScenarioFile,
    pop_service: &PopService,
    postgres_container: &Option<ContainerAsync<Postgres>>,
    kafka_container: &Option<ContainerAsync<Kafka>>,
    keycloak_container: &Option<ContainerAsync<GenericImage>>,
    pop_server_container: &Option<ContainerAsync<GenericImage>>,
) -> HashMap<String, String> {
    let mut with_env = HashMap::new();

    setup_postgres_env(
        scenario_file,
        pop_service,
        postgres_container,
        &mut with_env,
    )
    .await;
    setup_kafka_env(scenario_file, pop_service, kafka_container, &mut with_env).await;
    setup_keycloak_env(
        scenario_file,
        pop_service,
        keycloak_container,
        &mut with_env,
    )
    .await;
    setup_pop_server_env(
        scenario_file,
        pop_service,
        pop_server_container,
        &mut with_env,
    )
    .await;

    with_env
}
