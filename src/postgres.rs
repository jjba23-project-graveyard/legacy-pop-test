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

use crate::pop_model::PopPostgres;
use testcontainers::ContainerAsync;
use testcontainers::{runners::AsyncRunner, ImageExt};
use testcontainers_modules::postgres::Postgres;

pub async fn start_postgres_container(
    pop_postgres: PopPostgres,
) -> Result<ContainerAsync<Postgres>, String> {
    Postgres::default()
        .with_name(pop_postgres.image_name.unwrap_or("postgres".to_string()))
        .with_tag(pop_postgres.image_tag.unwrap_or("17.2".to_string()))
        .start()
        .await
        .map_err(|e| format!("{}", e))
}
