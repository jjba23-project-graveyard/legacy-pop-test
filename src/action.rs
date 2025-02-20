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

use crate::http::{do_http_assertion, do_http_call};
use crate::kafka::{do_kafka_check, do_kafka_produce};

use crate::pop_model::{PopAction, PopEnv, PopKeycloak, PopService};

use crate::postgres::do_postgres_assertion;
use crate::sqlite::{do_sqlite_action, do_sqlite_assertion};

use log::info;

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
