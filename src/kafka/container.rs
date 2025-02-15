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

use crate::pop_model::{PopKafka, PopScenario, PopService};
use kafka::client::KafkaClient;
use kafka::producer::{Producer, Record, RequiredAcks};
use log::info;
use std::collections::HashMap;
use std::time::Duration;
use testcontainers::ContainerAsync;
use testcontainers::{runners::AsyncRunner, ImageExt};
use testcontainers_modules::kafka::{Kafka, KAFKA_PORT};
use tokio::fs;

// pub async fn check_topic(
//     bootstrap_server: String,
//     group: String,
//     topic: String,
// ) -> Result<(), String> {
//     let mut consumer = Consumer::from_hosts(vec![bootstrap_server])
//         .with_fallback_offset(FetchOffset::Earliest)
//         .with_group(group)
//         .with_topic(topic)
//         .with_offset_storage(Some(GroupOffsetStorage::Kafka))
//         .create()
//         .map_err(|e| e.to_string())?;

//     loop {
//         for ms in consumer.poll().unwrap().iter() {
//             for m in ms.messages() {
//                 println!("{:?}", m);
//             }
//             consumer.consume_messageset(ms);
//         }
//         consumer.commit_consumed().unwrap();
//     }
// }

pub async fn check_consumer_group_offsets(bootstrap_server: String) -> Result<(), String> {
    let mut client = KafkaClient::new(vec![bootstrap_server]);
    client.load_metadata_all().map_err(|e| e.to_string())?;
    let topics: Vec<String> = client.topics().names().map(ToOwned::to_owned).collect();
    info!("kafka topics: {:#?}", topics);
    let offsets = client
        .fetch_offsets(&topics, kafka::client::FetchOffset::Latest)
        .map_err(|e| e.to_string())?;
    info!("kafka offsets: {:#?}", offsets);
    Ok(())
}

pub async fn produce_kafka_message(
    bootstrap_server: String,
    file_path: String,
    topic: String,
) -> Result<(), String> {
    let mut producer = Producer::from_hosts(vec![bootstrap_server])
        .with_ack_timeout(Duration::from_secs(1))
        .with_required_acks(RequiredAcks::One)
        .create()
        .map_err(|e| e.to_string())?;

    let d = fs::read_to_string(file_path)
        .await
        .map_err(|e| e.to_string())?;

    producer
        .send(&Record::from_value(topic.as_str(), d.as_bytes()))
        .map_err(|e| e.to_string())
}

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
    scenario: &PopScenario,
) -> Result<Option<ContainerAsync<Kafka>>, String> {
    match &scenario.pop_kafka {
        Some(kafka) if kafka.enabled == Some(true) => {
            info!("start setting up kafka");
            Ok(Some(run_kafka_container(kafka.clone()).await.unwrap()))
        }
        _ => Ok(None),
    }
}

pub async fn setup_kafka_env(
    scenario: &PopScenario,
    pop_service: &PopService,
    kafka_container: &Option<ContainerAsync<Kafka>>,
    with_env: &mut HashMap<String, String>,
) {
    if scenario
        .pop_kafka
        .as_ref()
        .and_then(|k| k.enabled)
        .unwrap_or(false)
        && pop_service
            .kafka
            .as_ref()
            .and_then(|k| k.enabled)
            .unwrap_or(false)
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
