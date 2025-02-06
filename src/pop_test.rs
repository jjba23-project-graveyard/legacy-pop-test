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

use crate::checks::perform_checks;
use crate::health_check::health_check;
use crate::kafka::start_kafka_container;
use crate::pop_model::{PopScenarioFile, PopService};
use crate::postgres::start_postgres_container;

use paris::{error, info, success};
use std::collections::HashMap;
use std::{env, process::Stdio};
use testcontainers::ContainerAsync;
use testcontainers_modules::kafka::Kafka;

use testcontainers_modules::postgres::Postgres;
use tokio::process::{Child, Command};
use tokio::task::JoinSet;

pub async fn do_work(
    pop_service: PopService,
    with_env: HashMap<String, String>,
) -> Result<Child, std::io::Error> {
    info!("Task is running...");
    let user_home = env::var("HOME").unwrap();

    let dir = str::replace(&pop_service.dir, "$HOME", &user_home);

    info!("in dir {}", &dir);
    info!("with args {:?}", &pop_service.start_cmd_args);
    info!("with env {:?}", &with_env);

    let mut arg: Vec<String> = Vec::new();
    arg.push("-c".to_string());
    for b in &pop_service.start_cmd_args {
        arg.push(b.to_string());
    }

    info!("run shell with args {:?}", &arg);

    let b = &pop_service
        .shell_path
        .clone()
        .unwrap_or_else(|| "bash".to_string());

    let shell_path = str::replace(b, "$HOME", &user_home);
    let env: HashMap<String, String> = pop_service.env_vars.into_iter().chain(with_env).collect();
    Command::new(shell_path)
        .args(&arg)
        .current_dir(dir)
        .envs(env)
        .stdout(Stdio::inherit())
        .spawn()
}

pub async fn start_p(
    pop_service: PopService,
    with_env: HashMap<String, String>,
) -> Result<Child, std::io::Error> {
    do_work(pop_service, with_env).await
}

pub async fn start_h(pop_service: PopService) {
    let pp = pop_service.clone();
    let _ = health_check(pp.id, pp.base_url, pp.health_check).await;
}

pub async fn play_scenario(scenario_file: PopScenarioFile) -> Result<(), String> {
    let mut h_set = JoinSet::new();
    let mut children = Vec::new();

    info!("start setting up dependencies");

    let postgres_container: Option<ContainerAsync<Postgres>> =
        match &scenario_file.pop_test.pop_postgres {
            Some(postgres) => {
                info!("start setting up postgres");
                Some(start_postgres_container(postgres.clone()).await.unwrap())
            }
            _ => None,
        };

    let kafka_container: Option<ContainerAsync<Kafka>> = match &scenario_file.pop_test.pop_kafka {
        Some(kafka) => {
            if kafka.enabled == Some(true) {
                info!("start setting up kafka");
                Some(start_kafka_container(kafka.clone()).await.unwrap())
            } else {
                None
            }
        }
        _ => None,
    };

    for pop_service in scenario_file.pop_test.pop_services.clone() {
        info!("registering service: {}", pop_service.id);
        h_set.spawn(start_h(pop_service.clone()));

        let mut with_env = HashMap::from([]);

        if scenario_file
            .clone()
            .pop_test
            .pop_postgres
            .map(|p| p.enabled)
            == Some(Some(true))
            && pop_service
                .postgres
                .clone()
                .map(|p| p.enabled)
                .unwrap_or(Some(false))
                == Some(true)
        {
            let postgres_host = if let Some(p) = &postgres_container {
                Some(p.get_host().await.unwrap().to_string())
            } else {
                None
            };

            let postgres_port: i32 = if let Some(p) = &postgres_container {
                p.ports()
                    .await
                    .unwrap()
                    .map_to_host_port_ipv4(5432)
                    .unwrap_or(5432)
                    .into()
            } else {
                5432
            };

            if let Some(h) = postgres_host {
                with_env.insert(
                    pop_service
                        .postgres
                        .as_ref()
                        .and_then(|pp| pp.db_var.clone())
                        .unwrap_or("POSTGRES_DB".to_string()),
                    "postgres".to_string(),
                );
                with_env.insert(
                    pop_service
                        .postgres
                        .as_ref()
                        .and_then(|pp| pp.user_var.clone())
                        .unwrap_or("POSTGRES_USER".to_string()),
                    "postgres".to_string(),
                );
                with_env.insert(
                    pop_service
                        .postgres
                        .as_ref()
                        .and_then(|pp| pp.pass_var.clone())
                        .unwrap_or("POSTGRES_PASSWORD".to_string()),
                    "postgres".to_string(),
                );
                with_env.insert(
                    pop_service
                        .postgres
                        .as_ref()
                        .and_then(|pp| pp.port_var.clone())
                        .unwrap_or("POSTGRES_PORT".to_string()),
                    postgres_port.to_string(),
                );
                with_env.insert(
                    pop_service
                        .postgres
                        .as_ref()
                        .and_then(|pp| pp.host_var.clone())
                        .unwrap_or("POSTGRES_HOST".to_string()),
                    h.clone(),
                );
            }
        }

        if scenario_file.clone().pop_test.pop_kafka.map(|k| k.enabled) == Some(Some(true))
            && pop_service
                .kafka
                .clone()
                .map(|k| k.enabled)
                .unwrap_or(Some(false))
                == Some(true)
        {
            let kafka_host = if let Some(k) = &kafka_container {
                Some(k.get_host().await.unwrap().to_string())
            } else {
                None
            };

            let kafka_port: i32 = if let Some(k) = &kafka_container {
                k.ports()
                    .await
                    .unwrap()
                    .map_to_host_port_ipv4(9092)
                    .unwrap_or(9092)
                    .into()
            } else {
                9092
            };

            match kafka_host {
                Some(h) => with_env.insert(
                    pop_service
                        .kafka
                        .as_ref()
                        .and_then(|pp| pp.bootstrap_server_var.clone())
                        .unwrap_or("KAFKA_BOOTSTRAP_SERVER".to_string()),
                    format!("{}:{}", h, kafka_port),
                ),
                _ => None,
            };
        }

        let p = start_p(pop_service, with_env.clone())
            .await
            .map_err(|_| "Failed to start process")?;
        children.push(p);
    }

    info!("awaiting for all services to be healthy");
    h_set.join_all().await;

    info!("performing tests and assertions");
    perform_checks(&scenario_file).await;

    terminate_processes(children).await;

    success!("finished playing scenario");
    Ok(())
}

pub async fn terminate_processes(children: Vec<Child>) {
    info!("terminating all processes");
    for mut child in children {
        info!("terminating pid: {:?}", child.id());
        let _ = child.kill().await;
    }
}

pub async fn do_with_file_data(user_cmd: String, file_data: String) -> Result<(), String> {
    info!("parsing YAML from file");
    let scenario_file: PopScenarioFile = serde_yml::from_str(&file_data).unwrap();

    success!("parsed pop scenario properly from YAML file");
    if user_cmd == "play-scenario" {
        play_scenario(scenario_file).await
    } else {
        let msg = "could not understand the wanted task to execute";
        error!("{}", msg);
        Err(msg.to_string())
    }
}
