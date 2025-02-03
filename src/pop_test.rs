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

use crate::pop_model::{PopScenario, PopService, PopServiceHealthCheck};
use core::{convert::TryInto, future::Future, time::Duration};
use paris::info;
use std::{
    collections::HashMap,
    env,
    process::{Command, Stdio},
};
use tokio::{task::JoinHandle, time::sleep};

pub async fn health_check(
    id: String,
    base_url: String,
    health_check: PopServiceHealthCheck,
) -> Result<(), &'static str> {
    info!("starting healthcheck for service: {}", id);
    match health_check {
        crate::pop_model::PopServiceHealthCheck::Http {
            path,
            patience_seconds,
        } => {
            let url = format!("{}{}", base_url, path);
            info!("calling health check URL: {}", url);

            let resp = reqwest::get(url)
                .await
                .unwrap()
                .json::<HashMap<String, String>>()
                .await;
            println!("{resp:#?}");
            sleep(Duration::from_secs(patience_seconds.try_into().unwrap())).await;
            Ok(())
        }
        crate::pop_model::PopServiceHealthCheck::Dummy { always_healthy } => {
            if always_healthy {
                Ok(())
            } else {
                Err("")
            }
        }
    }
}

pub async fn run_process(pop_service: PopService) -> Result<(), &'static str> {
    let user_home = env::var("HOME").unwrap();

    let dir = str::replace(&pop_service.dir, "$HOME", &user_home);

    info!("in dir {}", &dir);
    info!("with args {:?}", &pop_service.start_cmd_args);

    let mut arg: Vec<String> = Vec::new();
    arg.push("-c".to_string());
    for b in &pop_service.start_cmd_args {
        arg.push(b.to_string());
    }

    info!("bash with args {:?}", &arg);

    let b = &pop_service
        .bash_path
        .clone()
        .unwrap_or_else(|| "bash".to_string());

    let bash_path = str::replace(b, "$HOME", &user_home);
    let _ = Command::new(bash_path)
        .args(&arg)
        .current_dir(dir)
        .envs(&pop_service.env_vars)
        .stdout(Stdio::inherit())
        .output()
        .expect("failed to execute process");
    Ok(())
}

pub fn play_scenario(
    scenario: PopScenario,
) -> (
    HashMap<String, JoinHandle<impl Future<Output = Result<(), &'static str>>>>,
    HashMap<String, JoinHandle<impl Future<Output = Result<(), &'static str>>>>,
) {
    let mut process_tasks: HashMap<_, _> = HashMap::new();
    let mut health_tasks: HashMap<_, _> = HashMap::new();

    for pop_service in scenario.pop_services {
        info!("registering service: {}", pop_service.id);

        let p = pop_service.clone();
        let pk = format!("{}-process", &p.id);
        let process_task = tokio::spawn(async move { run_process(p) });
        process_tasks.insert(pk, process_task);

        let pp = pop_service.clone();
        let hk = format!("{}-health", &pp.id);
        let health_check_task =
            tokio::spawn(async move { health_check(pp.id, pp.base_url, pp.health_check) });
        health_tasks.insert(hk, health_check_task);
    }

    (process_tasks, health_tasks)
}
