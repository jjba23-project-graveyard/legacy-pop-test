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

use crate::http::{run_http_action, run_http_assertion};
use crate::kafka::container::{check_consumer_group_offsets, produce_kafka_message};
use crate::keycloak::client::{
    KeycloakClientId, KeycloakClientSecret, KeycloakPassword, KeycloakRealm, KeycloakUsername,
};
use crate::pop_model::{PopAction, PopEnv, PopKeycloak, PopService};
use crate::row_dsl::{get_row_expression_quantifier, make_checker_from_expression};
use log::{error, info};
use std::time::Duration;
use tokio::time::sleep;
use tokio_postgres::NoTls;

pub async fn run_action(
    pop_env: &PopEnv,
    services: &[PopService],
    action: &PopAction,
    keycloak: Option<PopKeycloak>,
) -> Result<(), String> {
    info!("executing action: {:#?}", action);
    match action {
        PopAction::KafkaCheck {
            name,
            delay_seconds,
            kafka_topic,
            check,
        } => {
            name.clone().inspect(|n| info!("starting action: {}", n));
            if let Some(delay) = delay_seconds {
                sleep(Duration::from_secs(*delay)).await;
            }

            match check.as_str() {
                "check-offsets" => {
                    check_consumer_group_offsets(
                        pop_env
                            .kafka_bootstrap_server
                            .clone()
                            .unwrap_or("".to_owned()),
                    )
                    .await
                }
                _ => Err("did not find a valid check to perform".to_string()),
            }
        }

        PopAction::PostgresAssertion {
            name,
            postgres_query,
            assert_row_count,
            assert_expressions,
            delay_seconds,
        } => {
            name.clone().inspect(|n| info!("starting action: {}", n));
            if let Some(delay) = delay_seconds {
                sleep(Duration::from_secs(*delay)).await;
            }

            let (client, connection) = tokio_postgres::connect(
                pop_env
                    .postgres_conn_string
                    .clone()
                    .unwrap_or("".to_string())
                    .as_str(),
                NoTls,
            )
            .await
            .map_err(|e| e.to_string())?;

            // The connection object performs the actual communication with the database,
            // so spawn it off to run on its own.
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    error!("connection error: {}", e);
                }
            });

            //.query("SELECT $1::TEXT", &[&"hello world"])

            let rows = client
                .query(postgres_query, &[])
                .await
                .map_err(|e| e.to_string())?;

            info!("postgres rows: {:#?}", rows);

            if let Some(rc) = assert_row_count {
                if rows.len() != (rc.to_owned() as usize) {
                    error!("found an unexpected amount of rows in the table")
                } else {
                    info!("found expected amount of rows in the table")
                }
            }

            for assert_expression in assert_expressions.clone().unwrap_or(Vec::new()) {
                let q = get_row_expression_quantifier(assert_expression.clone())?;
                info!("row expression quantifier: {:#?}", q);

                let row_checker = make_checker_from_expression(assert_expression.clone())?;

                match q {
                    crate::row_dsl::RowExpressionQuantifier::All => {
                        info!("TODO");
                        for row in &rows {
                            let is_valid_row = row_checker(row);
                            info!("is valid row {}, {:#?}", is_valid_row, row);
                        }
                    }
                    crate::row_dsl::RowExpressionQuantifier::First => {
                        if !rows.is_empty() {
                            info!("reading first row: {:#?}", rows[0]);
                            let is_valid_row = row_checker(&rows[0]);
                            info!("is valid row {}, {:#?}", is_valid_row, &rows[0]);
                        } else {
                            error!("no rows were found for your query")
                        }
                    }
                    crate::row_dsl::RowExpressionQuantifier::Last => {
                        let pos = rows.len();
                        info!("reading last row: {:#?}", rows[pos]);
                        let is_valid_row = row_checker(&rows[pos]);
                        info!("is valid row {}, {:#?}", is_valid_row, &rows[pos]);
                    }
                    crate::row_dsl::RowExpressionQuantifier::Position(p) => {
                        let pos = p as usize;
                        info!("reading row at pos {}: {:#?}", pos, rows[pos]);
                        let is_valid_row = row_checker(&rows[pos]);
                        info!("is valid row {}, {:#?}", is_valid_row, &rows[pos]);
                    }
                };
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
                pop_env
                    .kafka_bootstrap_server
                    .clone()
                    .unwrap_or("".to_owned()),
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
                info!("found service to check: {:#?}", service);
                info!("running http assertion: {:#?}", action);
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
            html_body_patterns,
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
                info!("found service to check: {:#?}", service);
                info!("running http assertion: {:#?}", action);
                run_http_assertion(
                    *assert_code,
                    assert_url.clone(),
                    service.base_url.clone(),
                    json_body_patterns.clone(),
                    html_body_patterns.clone(),
                    tok,
                )
                .await
            } else {
                Err("could not find service details for check".to_string())
            }
        }
    }
}
