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

use crate::pop_model::PopServer;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage};

pub async fn start_pop_server_container(
    pop_server: PopServer,
) -> Result<ContainerAsync<GenericImage>, String> {
    GenericImage::new(
        pop_server
            .image_name
            .unwrap_or("jjba23/pop-server".to_string()),
        pop_server.image_tag.unwrap_or("0.2.0".to_string()),
    )
    .start()
    .await
    .map_err(|e| format!("{}", e))
}
