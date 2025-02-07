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

use std::collections::HashMap;

use crate::pop_model::{PopPostgres, PopScenarioFile, PopService};
use paris::info;
use testcontainers::ContainerAsync;
use testcontainers::{runners::AsyncRunner, ImageExt};
use testcontainers_modules::postgres::Postgres;

async fn run_postgres_container(
    pop_postgres: PopPostgres,
) -> Result<ContainerAsync<Postgres>, String> {
    Postgres::default()
        .with_name(pop_postgres.image_name.unwrap_or("postgres".to_string()))
        .with_tag(pop_postgres.image_tag.unwrap_or("17.2".to_string()))
        .start()
        .await
        .map_err(|e| format!("{}", e))
}

pub async fn maybe_start_postgres(
    scenario_file: &PopScenarioFile,
) -> Result<Option<ContainerAsync<Postgres>>, String> {
    match &scenario_file.pop_test.pop_postgres {
        Some(postgres) => {
            info!("start setting up postgres");
            Ok(Some(
                run_postgres_container(postgres.clone()).await.unwrap(),
            ))
        }
        _ => Ok(None),
    }
}

pub async fn setup_postgres_env(
    scenario_file: &PopScenarioFile,
    pop_service: &PopService,
    postgres_container: &Option<ContainerAsync<Postgres>>,
    with_env: &mut HashMap<String, String>,
) {
    if scenario_file
        .pop_test
        .pop_postgres
        .as_ref()
        .map(|p| p.enabled)
        == Some(Some(true))
        && pop_service
            .postgres
            .as_ref()
            .map(|p| p.enabled)
            .unwrap_or(Some(false))
            == Some(true)
    {
        if let Some(p) = postgres_container {
            let host = p.get_host().await.unwrap().to_string();
            let port = p
                .ports()
                .await
                .unwrap()
                .map_to_host_port_ipv4(5432)
                .unwrap_or(5432);
            let postgres = pop_service.postgres.as_ref().unwrap();

            with_env.insert(
                postgres
                    .db_var
                    .clone()
                    .unwrap_or_else(|| "POSTGRES_DB".to_string()),
                "postgres".to_string(),
            );
            with_env.insert(
                postgres
                    .user_var
                    .clone()
                    .unwrap_or_else(|| "POSTGRES_USER".to_string()),
                "postgres".to_string(),
            );
            with_env.insert(
                postgres
                    .pass_var
                    .clone()
                    .unwrap_or_else(|| "POSTGRES_PASSWORD".to_string()),
                "postgres".to_string(),
            );
            with_env.insert(
                postgres
                    .port_var
                    .clone()
                    .unwrap_or_else(|| "POSTGRES_PORT".to_string()),
                port.to_string(),
            );
            with_env.insert(
                postgres
                    .host_var
                    .clone()
                    .unwrap_or_else(|| "POSTGRES_HOST".to_string()),
                host,
            );
        }
    }
}
