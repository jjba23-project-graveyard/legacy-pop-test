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
use crate::kafka::{do_kafka_check, do_kafka_produce};
use crate::keycloak::client::{
    KeycloakClientId, KeycloakClientSecret, KeycloakPassword, KeycloakRealm, KeycloakUsername,
};
use crate::pop_model::{
    HttpAssertion, HttpCall, PopAction, PopEnv, PopKeycloak, PopService,
};

use crate::postgres::do_postgres_assertion;
use crate::sqlite::{do_sqlite_action, do_sqlite_assertion};
use hex::encode;
use hmac::{Hmac, Mac};
use log::info;

use sha1::Sha1;
use std::collections::HashMap;
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;

// Define an alias for HMAC-SHA1
type HmacSha1 = Hmac<Sha1>;

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

        let hs = http_call.extra_headers.clone().unwrap_or_default();
        let hss: HashMap<String, String> = hs
            .iter()
            .map(|(k, v)| {
                if v == "${BODY_SHA1}" {
                    let key = b"mysecret";

                    let mut mac =
                        HmacSha1::new_from_slice(key).expect("HMAC can take key of any size");

                    mac.update(&msg.clone().into_bytes());

                    let result = mac.finalize();
                    let code_bytes = result.into_bytes();

                    let hex_result = encode(code_bytes);

                    info!("HMAC-SHA1: {}", format!("sha1={}", hex_result));

                    (k.clone(), format!("sha1={}", hex_result.clone()))
                } else {
                    (k.clone(), v.clone())
                }
            })
            .collect();

        run_http_action(
            http_call.action_url.clone(),
            service.base_url.clone(),
            tok,
            http_call.method.to_string(),
            &hss,
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

        let msg: String = match (
            http_assertion.message.clone(),
            http_assertion.message_file_path.clone(),
        ) {
            (None, None) => "".to_string(),
            (Some(msg), None) => msg,
            (Some(msg), Some(_)) => msg,
            (None, Some(msg_file)) => fs::read_to_string(msg_file).await.unwrap_or("".to_string()),
        };

        let hs = http_assertion.extra_headers.clone().unwrap_or_default();
        let hss: HashMap<String, String> = hs
            .iter()
            .map(|(k, v)| {
                if v == "${BODY_SHA1}" {
                    let key = b"mysecret";

                    let mut mac =
                        HmacSha1::new_from_slice(key).expect("HMAC can take key of any size");

                    mac.update(&msg.clone().into_bytes());

                    let result = mac.finalize();
                    let code_bytes = result.into_bytes();

                    let hex_result = encode(code_bytes);

                    info!("HMAC-SHA1: {}", format!("sha1={}", hex_result));

                    (k.clone(), format!("sha1={}", hex_result.clone()))
                } else {
                    (k.clone(), v.clone())
                }
            })
            .collect();

        run_http_assertion(
            http_assertion.assert_code,
            http_assertion.assert_url.clone(),
            service.base_url.clone(),
            http_assertion.json_patterns.clone(),
            http_assertion.html_patterns.clone(),
            tok,
            http_assertion.method.to_string(),
            &hss,
            msg,
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
