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

use log::{debug, info};
use testcontainers::{ContainerAsync, Image};
use tokio::{io::AsyncBufReadExt, task::JoinHandle};

pub async fn maybe_terminate_container<T>(container: Option<ContainerAsync<T>>)
where
    T: Image,
{
    if let Some(c) = container {
        let id = c.id();
        info!("removing container: {}", id);
        let _ = c.rm().await;
    }
}

pub async fn setup_container_logging<T>(
    container: &ContainerAsync<T>,
) -> (JoinHandle<()>, JoinHandle<()>)
where
    T: Image,
{
    let stdout = container.stdout(true);
    let mut stdout_lines = stdout.lines();
    let stderr = container.stderr(true);
    let mut stderr_lines = stderr.lines();

    let stdout_task = tokio::spawn(async move {
        while let Ok(result) = stdout_lines.next_line().await {
            if let Some(r) = result {
                debug!("{}", r);
            }
        }
    });

    let stderr_task = tokio::spawn(async move {
        while let Ok(result) = stderr_lines.next_line().await {
            if let Some(r) = result {
                debug!("{}", r);
            }
        }
    });

    (stdout_task, stderr_task)
}
