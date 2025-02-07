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

use crate::pop_model::{PopKafka, PopScenarioFile, PopService};
use paris::info;
use testcontainers::ContainerAsync;
use testcontainers::{runners::AsyncRunner, ImageExt};
use testcontainers_modules::kafka::{Kafka, KAFKA_PORT};

async fn run_kafka_container(pop_kafka: PopKafka) -> Result<ContainerAsync<Kafka>, String> {
    Kafka::default()
        .with_name(
            pop_kafka
                .image_name
                .unwrap_or("confluentinc/cp-kafka".to_string()),
        )
        .with_tag(pop_kafka.image_tag.unwrap_or("6.1.1".to_string()))
        .start()
        .await
        .map_err(|e| format!("{}", e))
}

pub async fn maybe_start_kafka(
    scenario_file: &PopScenarioFile,
) -> Result<Option<ContainerAsync<Kafka>>, String> {
    match &scenario_file.pop_test.pop_kafka {
        Some(kafka) if kafka.enabled == Some(true) => {
            info!("start setting up kafka");
            Ok(Some(run_kafka_container(kafka.clone()).await.unwrap()))
        }
        _ => Ok(None),
    }
}

pub async fn setup_kafka_env(
    scenario_file: &PopScenarioFile,
    pop_service: &PopService,
    kafka_container: &Option<ContainerAsync<Kafka>>,
    with_env: &mut HashMap<String, String>,
) {
    if scenario_file.pop_test.pop_kafka.as_ref().map(|k| k.enabled) == Some(Some(true))
        && pop_service
            .kafka
            .as_ref()
            .map(|k| k.enabled)
            .unwrap_or(Some(false))
            == Some(true)
    {
        if let Some(k) = kafka_container {
            let host = k.get_host().await.unwrap().to_string();
            let port = k
                .ports()
                .await
                .unwrap()
                .map_to_host_port_ipv4(KAFKA_PORT)
                .unwrap_or(KAFKA_PORT.as_u16());
            let k: String = pop_service
                .kafka
                .as_ref()
                .unwrap()
                .bootstrap_server_var
                .clone()
                .unwrap_or("KAFKA_BOOTSTRAP_SERVER".to_string());

            with_env.insert(k, format!("{}:{}", host, port));
        }
    }
}
