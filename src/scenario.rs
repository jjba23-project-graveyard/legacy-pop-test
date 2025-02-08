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

use crate::actions::perform_checks;
use crate::health_check::start_health_check;
use crate::kafka::{maybe_start_kafka, setup_kafka_env};
use crate::keycloak::container::{maybe_start_keycloak, setup_keycloak_env};
use crate::pop_model::{PopScenarioFile, PopService};
use crate::pop_server::{maybe_start_pop_server, setup_pop_server_env};
use crate::postgres::container::{maybe_start_postgres, setup_postgres_env};
use crate::proc::{start_service, terminate_services};
use paris::{info, success};
use std::collections::HashMap;
use testcontainers::{ContainerAsync, GenericImage};
use testcontainers_modules::kafka::{Kafka, KAFKA_PORT};
use testcontainers_modules::postgres::Postgres;
use tokio::task::JoinSet;

pub async fn play_scenario(scenario_file: PopScenarioFile) -> Result<(), String> {
    let mut h_set = JoinSet::new();
    let mut children = Vec::new();

    info!("start setting up dependencies");

    let postgres_container = maybe_start_postgres(&scenario_file).await?;
    let pop_server_container = maybe_start_pop_server(&scenario_file).await?;
    let kafka_container = maybe_start_kafka(&scenario_file).await?;
    let keycloak_container = maybe_start_keycloak(&scenario_file).await?;

    for pop_service in scenario_file.pop_test.pop_services.clone() {
        info!("registering service: {}", pop_service.id);
        h_set.spawn(start_health_check(pop_service.clone()));

        let with_env = setup_environment(
            &scenario_file,
            &pop_service,
            &postgres_container,
            &kafka_container,
            &keycloak_container,
            &pop_server_container,
        )
        .await;

        let p = start_service(pop_service, with_env)
            .await
            .map_err(|_| "failed to start process")?;
        children.push(p);
    }

    info!("awaiting for all services to be healthy");
    h_set.join_all().await;

    info!("gathering env");
    let mut kafka_bootstrap_server = "".to_string();
    if let Some(k) = kafka_container {
        let host = k.get_host().await.unwrap().to_string();
        let port = k
            .ports()
            .await
            .unwrap()
            .map_to_host_port_ipv4(KAFKA_PORT)
            .unwrap_or(KAFKA_PORT.as_u16());

        let fmt = format!("{}:{}", host, port);
        kafka_bootstrap_server = fmt;
    }

    let pop_env = crate::pop_model::PopEnv {
        kafka_bootstrap_server: kafka_bootstrap_server.to_string(),
    };

    info!("{:?}", pop_env);

    info!("performing tests and assertions");
    perform_checks(&scenario_file, &pop_env).await;

    terminate_services(children).await;

    success!("finished playing scenario");
    Ok(())
}

async fn setup_environment(
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
