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

use crate::pop_model::PopKafka;
use testcontainers::ContainerAsync;
use testcontainers::{runners::AsyncRunner, ImageExt};
use testcontainers_modules::kafka::Kafka;

pub async fn start_kafka_container(pop_kafka: PopKafka) -> Result<ContainerAsync<Kafka>, String> {
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
