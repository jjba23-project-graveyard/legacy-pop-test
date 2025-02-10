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

use crate::environment::setup_environment;
use crate::health_check::start_health_check;
use crate::http::run_http_assertion;
use crate::kafka::container::{maybe_start_kafka, produce_kafka_message};
use crate::keycloak::container::maybe_start_keycloak;
use crate::pop_model::{PopAction, PopEnv, PopScenarioFile, PopService};
use crate::pop_server::maybe_start_pop_server;
use crate::postgres::container::maybe_start_postgres;
use crate::proc::{start_service, terminate_services};
use paris::{error, info, success};
use std::time::Duration;
use testcontainers::{ContainerAsync, Image};
use testcontainers_modules::kafka::KAFKA_PORT;
use tokio::task::JoinSet;
use tokio::time::sleep;

async fn maybe_terminate_container<T>(container: Option<ContainerAsync<T>>)
where
    T: Image,
{
    if let Some(c) = container {
        let id = c.id();
        info!("terminating and removing container: {}", id);
        let _ = c.stop().await;
        let _ = c.rm().await;
    }
}

pub async fn run_action(
    pop_env: &PopEnv,
    services: &[PopService],
    action: &PopAction,
) -> Result<(), String> {
    info!("executing action: {:?}", action);
    match action {
        PopAction::KafkaProduceFromFileAction {
            message_file_path,
            delay_seconds,
            kafka_topic,
        } => {
            if let Some(delay) = delay_seconds {
                sleep(Duration::from_secs(*delay)).await;
            }

            info!("producing kafka message");
            let r = produce_kafka_message(
                pop_env.kafka_bootstrap_server.clone(),
                message_file_path.to_owned(),
                kafka_topic.to_owned(),
            )
            .await;
            r.inspect_err(|e| error!("{}", e))
        }
        PopAction::HttpAssertion {
            assert_code,
            assert_url,
            service_id,
            delay_seconds,
            json_body_patterns,
        } => {
            if let Some(delay) = delay_seconds {
                sleep(Duration::from_secs(*delay)).await;
            }

            if let Some(service) = services.iter().find(|s| s.id == *service_id) {
                info!("found service to check: {:?}", service);
                info!("running http assertion: {:?}", action);
                run_http_assertion(
                    *assert_code,
                    assert_url.clone(),
                    service.base_url.clone(),
                    json_body_patterns.clone(),
                )
                .await
            } else {
                Err("could not find service details for check".to_string())
            }
        }
    }
}

pub async fn play_scenario(scenario_file: PopScenarioFile) -> Result<(), String> {
    let mut h_set = JoinSet::new();
    let mut children = Vec::new();

    info!("start setting up dependencies");

    let maybe_postgres_container = maybe_start_postgres(&scenario_file).await?;
    let maybe_pop_server_container = maybe_start_pop_server(&scenario_file).await?;
    let maybe_kafka_container = maybe_start_kafka(&scenario_file).await?;
    let maybe_keycloak_container = maybe_start_keycloak(&scenario_file).await?;

    for pop_service in scenario_file.pop_test.pop_services.clone() {
        info!("registering service: {}", pop_service.id);
        h_set.spawn(start_health_check(pop_service.clone()));

        let with_env = setup_environment(
            &scenario_file,
            &pop_service,
            &maybe_postgres_container,
            &maybe_kafka_container,
            &maybe_keycloak_container,
            &maybe_pop_server_container,
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
    if let Some(k) = &maybe_kafka_container {
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
    let services = &scenario_file.pop_test.pop_services;

    for action in &scenario_file
        .pop_test
        .pop_actions
        .clone()
        .unwrap_or_default()
    {
        run_action(&pop_env, services, action).await?
    }

    terminate_services(children).await;

    maybe_terminate_container(maybe_kafka_container).await;
    maybe_terminate_container(maybe_postgres_container).await;
    maybe_terminate_container(maybe_keycloak_container).await;
    maybe_terminate_container(maybe_pop_server_container).await;

    success!("finished playing scenario");
    Ok(())
}
