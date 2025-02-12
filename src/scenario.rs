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

use crate::containers::{maybe_terminate_container, setup_container_logging};
use crate::environment::setup_environment;
use crate::health_check::start_health_check;
use crate::http::{run_http_action, run_http_assertion};
use crate::kafka::container::{maybe_start_kafka, produce_kafka_message};
use crate::keycloak::client::{
    KeycloakClient, KeycloakClientId, KeycloakClientSecret, KeycloakPassword, KeycloakRealm,
    KeycloakUrl, KeycloakUsername,
};
use crate::keycloak::container::maybe_start_keycloak;
use crate::pop_model::{PopAction, PopEnv, PopKeycloak, PopScenario, PopService};
use crate::pop_server::maybe_start_pop_server;
use crate::postgres::container::maybe_start_postgres;
use crate::proc::{start_service, terminate_services};
use log::{error, info};
use std::time::Duration;

use testcontainers_modules::kafka::KAFKA_PORT;

use tokio::task::JoinSet;
use tokio::time::sleep;

pub async fn run_action(
    pop_env: &PopEnv,
    services: &[PopService],
    action: &PopAction,
    keycloak: Option<PopKeycloak>,
) -> Result<(), String> {
    info!("executing action: {:?}", action);
    match action {
        PopAction::PostgresAssertion {
            name,
            postgres_query,
            assert_row_count,
            assert_expression,
            delay_seconds,
        } => {
            name.clone().inspect(|n| info!("starting action: {}", n));
            if let Some(delay) = delay_seconds {
                sleep(Duration::from_secs(*delay)).await;
            }

            Ok(())
        }
        PopAction::KafkaProduceFromFileAction {
            name,
            message_file_path,
            delay_seconds,
            kafka_topic,
        } => {
            name.clone().inspect(|n| info!("starting action: {}", n));
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
        PopAction::HttpAction {
            name,
            action_url,
            service_id,
            delay_seconds,
            with_keycloak_user_token,
            method,
            message_file_path,
        } => {
            name.clone().inspect(|n| info!("starting action: {}", n));
            if let Some(delay) = delay_seconds {
                sleep(Duration::from_secs(*delay)).await;
            }

            let mut tok = None;
            if *with_keycloak_user_token == Some(true) {
                if let Some(mut kclient) = pop_env.keycloak_client.clone() {
                    if let Some(kk) = keycloak {
                        let maybe_tok = kclient
                            .get_user_token(
                                KeycloakRealm(kk.realm),
                                KeycloakClientId(kk.client_id),
                                KeycloakClientSecret(kk.client_secret),
                                KeycloakUsername(kk.username),
                                KeycloakPassword(kk.password),
                            )
                            .await;
                        let _ = maybe_tok.inspect(|t| tok = Some(t.clone().0));
                    }
                }
            }

            if let Some(service) = services.iter().find(|s| s.id == *service_id) {
                info!("found service to check: {:?}", service);
                info!("running http assertion: {:?}", action);
                run_http_action(
                    action_url.clone(),
                    service.base_url.clone(),
                    tok,
                    method.to_string(),
                    message_file_path.clone(),
                )
                .await
            } else {
                Err("could not find service details for check".to_string())
            }
        }
        PopAction::HttpAssertion {
            name,
            assert_code,
            assert_url,
            service_id,
            delay_seconds,
            json_body_patterns,
            with_keycloak_user_token,
        } => {
            name.clone().inspect(|n| info!("starting action: {}", n));
            if let Some(delay) = delay_seconds {
                sleep(Duration::from_secs(*delay)).await;
            }

            let mut tok = None;
            if *with_keycloak_user_token == Some(true) {
                if let Some(mut kclient) = pop_env.keycloak_client.clone() {
                    if let Some(kk) = keycloak {
                        let maybe_tok = kclient
                            .get_user_token(
                                KeycloakRealm(kk.realm),
                                KeycloakClientId(kk.client_id),
                                KeycloakClientSecret(kk.client_secret),
                                KeycloakUsername(kk.username),
                                KeycloakPassword(kk.password),
                            )
                            .await;
                        let _ = maybe_tok.inspect(|t| tok = Some(t.clone().0));
                    }
                }
            }

            if let Some(service) = services.iter().find(|s| s.id == *service_id) {
                info!("found service to check: {:?}", service);
                info!("running http assertion: {:?}", action);
                run_http_assertion(
                    *assert_code,
                    assert_url.clone(),
                    service.base_url.clone(),
                    json_body_patterns.clone(),
                    tok,
                )
                .await
            } else {
                Err("could not find service details for check".to_string())
            }
        }
    }
}

pub async fn play_scenario(scenario: PopScenario) -> Result<(), String> {
    let mut h_set = JoinSet::new();
    let mut children = Vec::new();

    info!("start setting up dependencies");

    let maybe_postgres_container = maybe_start_postgres(&scenario).await?;
    let postgres_logs = match maybe_postgres_container {
        Some(ref postgres_container) => {
            let o = setup_container_logging(postgres_container).await;
            Some(o)
        }
        None => None,
    };

    let maybe_pop_server_container = maybe_start_pop_server(&scenario).await?;
    let pop_server_logs = match maybe_pop_server_container {
        Some(ref pop_server_container) => {
            let o = setup_container_logging(pop_server_container).await;
            Some(o)
        }
        None => None,
    };

    let maybe_kafka_container = maybe_start_kafka(&scenario).await?;
    let kafka_logs = match maybe_kafka_container {
        Some(ref kafka_container) => {
            let o = setup_container_logging(kafka_container).await;
            Some(o)
        }
        None => None,
    };

    let maybe_keycloak_container = maybe_start_keycloak(&scenario).await?;
    let keycloak_logs = match maybe_keycloak_container {
        Some(ref keycloak_container) => {
            let o = setup_container_logging(keycloak_container).await;
            Some(o)
        }
        None => None,
    };

    for pop_service in scenario.pop_services.clone().unwrap_or_default() {
        info!("registering service: {}", pop_service.id);
        h_set.spawn(start_health_check(pop_service.clone()));

        let with_env = setup_environment(
            &scenario,
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

    let mut keycloak_client = None;
    if let Some(kc) = &maybe_keycloak_container {
        let host = kc.get_host().await.unwrap().to_string();
        let port = kc
            .ports()
            .await
            .unwrap()
            .map_to_host_port_ipv4(8080)
            .unwrap_or(8080);
        keycloak_client = Some(KeycloakClient {
            url: KeycloakUrl(format!("http://{}:{}", host, port)),
        });
    }

    let pop_env = crate::pop_model::PopEnv {
        kafka_bootstrap_server: kafka_bootstrap_server.to_string(),
        keycloak_client,
    };

    info!("{:?}", pop_env);

    info!("performing tests and assertions");
    let services = &scenario.pop_services.clone().unwrap_or_default();

    for action in &scenario.pop_actions.clone().unwrap_or_default() {
        let action_result =
            run_action(&pop_env, services, action, scenario.pop_keycloak.clone()).await;
        match action_result {
            Ok(_) => info!("ran action successfully: {:?}", action),
            Err(e) => error!("{}", e),
        }
    }

    terminate_services(children).await;

    if let Some((keycloak_out, keycloak_err)) = keycloak_logs {
        keycloak_out.abort();
        keycloak_err.abort();
    }
    maybe_terminate_container(maybe_keycloak_container).await;

    if let Some((kafka_out, kafka_err)) = kafka_logs {
        kafka_out.abort();
        kafka_err.abort();
    }
    maybe_terminate_container(maybe_kafka_container).await;

    if let Some((postgres_out, postgres_err)) = postgres_logs {
        postgres_out.abort();
        postgres_err.abort();
    }
    maybe_terminate_container(maybe_postgres_container).await;

    if let Some((pop_server_out, pop_server_err)) = pop_server_logs {
        pop_server_out.abort();
        pop_server_err.abort();
    }
    maybe_terminate_container(maybe_pop_server_container).await;

    info!("finished playing scenario");
    Ok(())
}
