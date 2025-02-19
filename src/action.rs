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
use crate::pop_model::{
    HttpAssertion, HttpCall, KafkaCheck, KafkaProduce, PopAction, PopEnv, PopKeycloak, PopService,
    PostgresAssertion, SqliteAction, SqliteAssertion,
};
use crate::postgres::checker::postgres_checker_from_expression;
use crate::row_dsl::get_row_expression_quantifier;
use crate::sqlite::sqlite_checker_from_expression;
use log::{debug, error, info};
use rusqlite::fallible_streaming_iterator::FallibleStreamingIterator;
use rusqlite::{Connection, Row};
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;
use tokio_postgres::NoTls;

pub async fn do_kafka_check(pop_env: &PopEnv, kafka_check: &KafkaCheck) -> Result<(), String> {
    if let Some(delay) = kafka_check.delay_seconds {
        sleep(Duration::from_secs(delay)).await;
    }

    kafka_check
        .name
        .clone()
        .inspect(|n| info!("starting action: {}", n));

    match kafka_check.check.as_str() {
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

pub async fn do_postgres_assertion(
    pop_env: &PopEnv,
    postgres_assertion: &PostgresAssertion,
) -> Result<(), String> {
    if let Some(delay) = postgres_assertion.delay_seconds {
        sleep(Duration::from_secs(delay)).await;
    }

    postgres_assertion
        .name
        .clone()
        .inspect(|n| info!("starting action: {}", n));

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

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });

    let rows = client
        .query(&postgres_assertion.postgres_query, &[])
        .await
        .map_err(|e| e.to_string())?;

    if let Some(rc) = postgres_assertion.assert_row_count {
        if rows.len() != (rc.to_owned() as usize) {
            error!("found an unexpected amount of rows in the table")
        } else {
            info!("found expected amount of rows in the table")
        }
    }

    for assert_expression in postgres_assertion
        .assert_expressions
        .clone()
        .unwrap_or_default()
    {
        let q = get_row_expression_quantifier(assert_expression.clone())?;
        info!("row expression quantifier: {:#?}", q);

        let row_checker = postgres_checker_from_expression(assert_expression.clone())?;

        match q {
            crate::row_dsl::RowExpressionQuantifier::All => {
                info!("TODO");
                for row in &rows {
                    let is_valid_row = row_checker(row);
                    info!("is valid row {}", is_valid_row);
                }
            }
            crate::row_dsl::RowExpressionQuantifier::First => {
                if !rows.is_empty() {
                    info!("reading first row");
                    let is_valid_row = row_checker(&rows[0]);
                    info!("is valid row {}", is_valid_row);
                } else {
                    error!("no rows were found for your query")
                }
            }
            crate::row_dsl::RowExpressionQuantifier::Last => {
                let pos = rows.len();
                info!("reading last row");
                let is_valid_row = row_checker(&rows[pos]);
                info!("is valid row {}", is_valid_row);
            }
            crate::row_dsl::RowExpressionQuantifier::Position(p) => {
                let pos = p as usize;
                info!("reading row at pos {}", pos);
                let is_valid_row = row_checker(&rows[pos]);
                info!("is valid row {}", is_valid_row);
            }
        };
    }

    Ok(())
}

fn extract_row(row: &Row, col_count: usize) -> Vec<String> {
    (0..col_count)
        .map(|i| {
            row.get::<usize, String>(i)
                .unwrap_or_else(|_| "".to_string())
        })
        .collect()
}

pub async fn do_sqlite_assertion(
    pop_env: &PopEnv,
    sqlite_assertion: &SqliteAssertion,
) -> Result<(), String> {
    if let Some(delay) = sqlite_assertion.delay_seconds {
        sleep(Duration::from_secs(delay)).await;
    }

    sqlite_assertion
        .name
        .clone()
        .inspect(|n| info!("starting action: {}", n));

    let conn = Connection::open(
        pop_env
            .sqlite_db_path
            .clone()
            .unwrap_or("./pop-test.sqlite".to_string()),
    )
    .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(&sqlite_assertion.sqlite_query)
        .map_err(|e| e.to_string())?;

    let col_co = stmt.column_count();
    let cc = 0..col_co;
    let real_rc = stmt
        .query_map([], |ro| Ok(extract_row(ro, col_co)))
        .map_err(|e| e.to_string())?
        .count();

    if let Some(rc) = sqlite_assertion.assert_row_count {
        if real_rc != (rc.to_owned() as usize) {
            error!("found an unexpected amount of rows in the table")
        } else {
            info!("found expected amount of rows in the table")
        }
    }

    let mut xx = stmt
        .query_map([], |ro| Ok(extract_row(ro, col_co)))
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    while let Some(Ok(v)) = xx.next() {
        results.push(v);
    }

    info!("row results: {:?}", results);

    for assert_expression in sqlite_assertion
        .assert_expressions
        .clone()
        .unwrap_or_default()
    {
        let rc = results.clone();
        let q = get_row_expression_quantifier(assert_expression.clone())?;
        info!("row expression quantifier: {:#?}", q);

        info!("assert: {}", assert_expression);

        let row_checker = sqlite_checker_from_expression(assert_expression.clone())?;

        match q {
            crate::row_dsl::RowExpressionQuantifier::First => {
                if real_rc != 0_usize {
                    info!("reading first row");
                    let is_valid_row = row_checker(rc[0].clone());
                    info!("is valid row {}", is_valid_row);
                } else {
                    error!("no rows were found for your query")
                }
            }
            _ => info!(""),
        };
    }

    Ok(())
}

pub async fn do_sqlite_action(
    pop_env: &PopEnv,
    sqlite_action: &SqliteAction,
) -> Result<(), String> {
    if let Some(delay) = sqlite_action.delay_seconds {
        sleep(Duration::from_secs(delay)).await;
    }

    sqlite_action
        .name
        .clone()
        .inspect(|n| info!("starting action: {}", n));

    info!(
        "{}",
        pop_env
            .sqlite_db_path
            .clone()
            .unwrap_or("./pop-test.sqlite".to_string())
    );

    let conn = Connection::open(
        pop_env
            .sqlite_db_path
            .clone()
            .unwrap_or("./pop-test.sqlite".to_string()),
    )
    .map_err(|e| e.to_string())?;

    let qs: Vec<String> = sqlite_action
        .sqlite_action
        .split(";")
        .map(|s| {
            let ss = s.replace("\n", " ");
            ss.clone().trim().to_string()
        })
        .filter(|s| !s.is_empty())
        .collect();

    debug!("query vec: {:?}", qs);

    for q in qs {
        debug!("query: {}", q);
        let r = conn.execute(q.as_str(), []).map_err(|e| e.to_string())?;
        debug!("result: {}", r);
    }

    Ok(())
}

pub async fn do_kafka_produce(
    pop_env: &PopEnv,
    kafka_produce: &KafkaProduce,
) -> Result<(), String> {
    if let Some(delay) = kafka_produce.delay_seconds {
        sleep(Duration::from_secs(delay)).await;
    }

    kafka_produce
        .name
        .clone()
        .inspect(|n| info!("starting action: {}", n));

    let msg: String = match (
        kafka_produce.message.clone(),
        kafka_produce.message_file_path.clone(),
    ) {
        (None, None) => "".to_string(),
        (Some(msg), None) => msg,
        (Some(msg), Some(_)) => msg,
        (None, Some(msg_file)) => fs::read_to_string(msg_file).await.unwrap_or("".to_string()),
    };

    info!("producing kafka message: {}", msg);

    let r = produce_kafka_message(
        pop_env
            .kafka_bootstrap_server
            .clone()
            .unwrap_or("".to_owned()),
        msg,
        kafka_produce.kafka_topic.to_owned(),
    )
    .await;
    r.inspect_err(|e| error!("{}", e))
}

pub async fn do_http_call(
    pop_env: &PopEnv,
    services: &[PopService],
    keycloak: Option<PopKeycloak>,
    http_call: &HttpCall,
) -> Result<(), String> {
    if let Some(delay) = http_call.delay_seconds {
        sleep(Duration::from_secs(delay)).await;
    }

    http_call
        .name
        .clone()
        .inspect(|n| info!("starting action: {}", n));

    let mut tok = None;
    if http_call.with_keycloak_user_token == Some(true) {
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

    if let Some(service) = services.iter().find(|s| s.id == http_call.service_id) {
        info!("found service to check: {:#?}", service);
        info!("running http assertion: {:#?}", http_call);

        let msg: String = match (
            http_call.message.clone(),
            http_call.message_file_path.clone(),
        ) {
            (None, None) => "".to_string(),
            (Some(msg), None) => msg,
            (Some(msg), Some(_)) => msg,
            (None, Some(msg_file)) => fs::read_to_string(msg_file).await.unwrap_or("".to_string()),
        };

        run_http_action(
            http_call.action_url.clone(),
            service.base_url.clone(),
            tok,
            http_call.method.to_string(),
            msg,
        )
        .await
    } else {
        Err("could not find service details for check".to_string())
    }
}

pub async fn do_http_assertion(
    pop_env: &PopEnv,
    services: &[PopService],
    keycloak: Option<PopKeycloak>,
    http_assertion: &HttpAssertion,
) -> Result<(), String> {
    if let Some(delay) = http_assertion.delay_seconds {
        sleep(Duration::from_secs(delay)).await;
    }

    http_assertion
        .name
        .clone()
        .inspect(|n| info!("starting action: {}", n));

    let mut tok = None;
    if http_assertion.with_keycloak_user_token == Some(true) {
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

    if let Some(service) = services.iter().find(|s| s.id == http_assertion.service_id) {
        info!("running http assertion: {:#?}", http_assertion);
        run_http_assertion(
            http_assertion.assert_code,
            http_assertion.assert_url.clone(),
            service.base_url.clone(),
            http_assertion.json_patterns.clone(),
            http_assertion.html_patterns.clone(),
            tok,
        )
        .await
    } else {
        Err("could not find service details for check".to_string())
    }
}

pub async fn run_action(
    pop_env: &PopEnv,
    services: &[PopService],
    action: &PopAction,
    keycloak: Option<PopKeycloak>,
) -> Result<(), String> {
    info!("executing action: {:#?}", action);
    match action {
        PopAction::KafkaCheck(kafka_check) => do_kafka_check(pop_env, kafka_check).await,
        PopAction::PostgresAssertion(postgres_assertion) => {
            do_postgres_assertion(pop_env, postgres_assertion).await
        }
        PopAction::SqliteAssertion(sqlite_assertion) => {
            do_sqlite_assertion(pop_env, sqlite_assertion).await
        }
        PopAction::SqliteAction(sqlite_action) => do_sqlite_action(pop_env, sqlite_action).await,
        PopAction::KafkaProduce(kafka_produce) => do_kafka_produce(pop_env, kafka_produce).await,
        PopAction::HttpCall(http_call) => {
            do_http_call(pop_env, services, keycloak, http_call).await
        }
        PopAction::HttpAssertion(http_assertion) => {
            do_http_assertion(pop_env, services, keycloak, http_assertion).await
        }
    }
}
