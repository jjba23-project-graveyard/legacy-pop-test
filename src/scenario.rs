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

use crate::checks::perform_checks;
use crate::health_check::start_health_check;
use crate::kafka::start_kafka_container;
use crate::pop_model::PopScenarioFile;
use crate::pop_server::start_pop_server_container;
use crate::postgres::start_postgres_container;
use crate::proc::{start_service, terminate_services};

use paris::{info, success};
use std::collections::HashMap;
use testcontainers::{ContainerAsync, GenericImage};
use testcontainers_modules::kafka::Kafka;

use testcontainers_modules::postgres::Postgres;
use tokio::task::JoinSet;

pub async fn play_scenario(scenario_file: PopScenarioFile) -> Result<(), String> {
    let mut h_set = JoinSet::new();
    let mut children = Vec::new();

    info!("start setting up dependencies");

    let postgres_container: Option<ContainerAsync<Postgres>> =
        match &scenario_file.pop_test.pop_postgres {
            Some(postgres) => {
                info!("start setting up postgres");
                Some(start_postgres_container(postgres.clone()).await.unwrap())
            }
            _ => None,
        };

    let pop_server_container: Option<ContainerAsync<GenericImage>> =
        match &scenario_file.pop_test.pop_server {
            Some(pop_server) => {
                if pop_server.enabled == Some(true) {
                    info!("start setting up pop_server");
                    Some(
                        start_pop_server_container(pop_server.clone())
                            .await
                            .unwrap(),
                    )
                } else {
                    None
                }
            }
            _ => None,
        };

    let kafka_container: Option<ContainerAsync<Kafka>> = match &scenario_file.pop_test.pop_kafka {
        Some(kafka) => {
            if kafka.enabled == Some(true) {
                info!("start setting up kafka");
                Some(start_kafka_container(kafka.clone()).await.unwrap())
            } else {
                None
            }
        }
        _ => None,
    };

    for pop_service in scenario_file.pop_test.pop_services.clone() {
        info!("registering service: {}", pop_service.id);
        h_set.spawn(start_health_check(pop_service.clone()));

        let mut with_env = HashMap::from([]);

        if scenario_file
            .clone()
            .pop_test
            .pop_postgres
            .map(|p| p.enabled)
            == Some(Some(true))
            && pop_service
                .postgres
                .clone()
                .map(|p| p.enabled)
                .unwrap_or(Some(false))
                == Some(true)
        {
            let postgres_host = if let Some(p) = &postgres_container {
                Some(p.get_host().await.unwrap().to_string())
            } else {
                None
            };

            let postgres_port: u16 = if let Some(p) = &postgres_container {
                p.ports()
                    .await
                    .unwrap()
                    .map_to_host_port_ipv4(5432)
                    .unwrap_or(5432)
            } else {
                5432
            };

            if let Some(h) = postgres_host {
                with_env.insert(
                    pop_service
                        .postgres
                        .as_ref()
                        .and_then(|pp| pp.db_var.clone())
                        .unwrap_or("POSTGRES_DB".to_string()),
                    "postgres".to_string(),
                );
                with_env.insert(
                    pop_service
                        .postgres
                        .as_ref()
                        .and_then(|pp| pp.user_var.clone())
                        .unwrap_or("POSTGRES_USER".to_string()),
                    "postgres".to_string(),
                );
                with_env.insert(
                    pop_service
                        .postgres
                        .as_ref()
                        .and_then(|pp| pp.pass_var.clone())
                        .unwrap_or("POSTGRES_PASSWORD".to_string()),
                    "postgres".to_string(),
                );
                with_env.insert(
                    pop_service
                        .postgres
                        .as_ref()
                        .and_then(|pp| pp.port_var.clone())
                        .unwrap_or("POSTGRES_PORT".to_string()),
                    postgres_port.to_string(),
                );
                with_env.insert(
                    pop_service
                        .postgres
                        .as_ref()
                        .and_then(|pp| pp.host_var.clone())
                        .unwrap_or("POSTGRES_HOST".to_string()),
                    h.clone(),
                );
            }
        }

        if scenario_file.clone().pop_test.pop_kafka.map(|k| k.enabled) == Some(Some(true))
            && pop_service
                .kafka
                .clone()
                .map(|k| k.enabled)
                .unwrap_or(Some(false))
                == Some(true)
        {
            let kafka_host = if let Some(k) = &kafka_container {
                Some(k.get_host().await.unwrap().to_string())
            } else {
                None
            };

            let kafka_port: u16 = if let Some(k) = &kafka_container {
                k.ports()
                    .await
                    .unwrap()
                    .map_to_host_port_ipv4(9092)
                    .unwrap_or(9092)
            } else {
                9092
            };

            match kafka_host {
                Some(h) => with_env.insert(
                    pop_service
                        .kafka
                        .as_ref()
                        .and_then(|pp| pp.bootstrap_server_var.clone())
                        .unwrap_or("KAFKA_BOOTSTRAP_SERVER".to_string()),
                    format!("{}:{}", h, kafka_port),
                ),
                _ => None,
            };
        }

        if scenario_file.clone().pop_test.pop_server.map(|p| p.enabled) == Some(Some(true))
            && pop_service
                .pop_server
                .clone()
                .map(|p| p.enabled)
                .unwrap_or(Some(false))
                == Some(true)
        {
            let pop_server_host = if let Some(p) = &pop_server_container {
                Some(p.get_host().await.unwrap().to_string())
            } else {
                None
            };

            let pop_server_port: u16 = if let Some(p) = &pop_server_container {
                p.ports()
                    .await
                    .unwrap()
                    .map_to_host_port_ipv4(50088)
                    .unwrap_or(50088)
            } else {
                50088
            };

            if let Some(h) = pop_server_host {
                pop_service
                    .clone()
                    .pop_server
                    .unwrap()
                    .url_vars
                    .unwrap()
                    .into_iter()
                    .for_each(|v| {
                        with_env.insert(v, format!("http://{}:{}", h, pop_server_port));
                    });
            };
        }

        let p = start_service(pop_service, with_env.clone())
            .await
            .map_err(|_| "Failed to start process")?;
        children.push(p);
    }

    info!("awaiting for all services to be healthy");
    h_set.join_all().await;

    info!("performing tests and assertions");
    perform_checks(&scenario_file).await;

    terminate_services(children).await;

    success!("finished playing scenario");
    Ok(())
}
