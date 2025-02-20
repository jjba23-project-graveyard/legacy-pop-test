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

use container::{check_consumer_group_offsets, produce_kafka_message};
use tokio::time::sleep;

use crate::pop_model::{KafkaCheck, KafkaProduce, PopEnv};
use log::{error, info};
use std::time::Duration;
use tokio::fs;

pub mod container;

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
