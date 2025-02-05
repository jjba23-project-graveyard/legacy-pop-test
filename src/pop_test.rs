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

use crate::pop_check::check_pat;
use crate::pop_model::{
    BodyPattern, PopCheck, PopKafka, PopPostgres, PopScenarioFile, PopService,
    PopServiceHealthCheck,
};
use core::time::Duration;
use jql_runner::runner::raw;
use paris::{error, info, success};
use serde_json::json;
use std::collections::HashMap;
use std::{env, process::Stdio};
use testcontainers::ContainerAsync;
use testcontainers::{runners::AsyncRunner, ImageExt};
use testcontainers_modules::kafka::Kafka;

use testcontainers_modules::postgres::Postgres;
use tokio::task::JoinSet;
use tokio::{
    process::{Child, Command},
    time::sleep,
};

pub async fn do_with_http_check(
    expected_code: Option<i32>,
    check_url: String,
    service_url: String,
    json_body_patterns: Option<Vec<BodyPattern>>,
) -> Result<(), String> {
    let a = raw(r#""a","b""#, &json!({ "a": 1, "b": 2 }));
    info!("{:?}", a);

    let url = format!("{}{}", service_url, check_url);

    let maybe_response = reqwest::get(url.clone()).await;
    println!("{maybe_response:#?}");
    let wanted_status_code = expected_code.unwrap_or_else(|| 200);

    match maybe_response {
        Err(e) => error!("{}", e),
        Ok(resp) => {
            if (resp.status().as_u16() as i32) != wanted_status_code {
                return Err("the HTTP status did not match the expectation".to_string());
            } else {
                success!(
                    "the HTTP response contained the expected status: {:?}",
                    wanted_status_code
                )
            }

            let j = &resp.json().await;
            match j {
                Ok(body) => {
                    check_pat(body, json_body_patterns.unwrap_or_else(|| Vec::new())).unwrap()
                }
                _ => (),
            }
            ()
        }
    }

    Ok(())
}

pub async fn health_check(
    id: String,
    base_url: String,
    health_check: PopServiceHealthCheck,
) -> Result<(), String> {
    info!("starting healthcheck for service: {}", id);
    match health_check {
        crate::pop_model::PopServiceHealthCheck::Http {
            path,
            patience_seconds,
        } => {
            let mut elapsed = 0;
            let url = format!("{}{}", base_url, path);
            while elapsed <= patience_seconds {
                info!("calling health check URL ({}): {}", elapsed, url);
                let resp = reqwest::get(url.clone()).await;
                println!("{resp:#?}");
                match resp {
                    Err(e) => {
                        error!("{}", e);
                        sleep(Duration::from_secs(2)).await;
                        elapsed += 5;
                        continue;
                    }
                    Ok(v) => {
                        success!("{:?}", v);
                        break;
                    }
                }
            }

            Ok(())
        }
        crate::pop_model::PopServiceHealthCheck::Dummy { always_healthy } => {
            if always_healthy {
                Ok(())
            } else {
                Err("".to_string())
            }
        }
    }
}

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

pub async fn start_h(pop_service: PopService) -> () {
    let pp = pop_service.clone();
    let _ = health_check(pp.id, pp.base_url, pp.health_check).await;
}

pub async fn start_postgres(pop_postgres: PopPostgres) -> Result<ContainerAsync<Postgres>, String> {
    Postgres::default()
        .with_name(pop_postgres.image_name)
        .with_tag(pop_postgres.image_tag)
        .start()
        .await
        .map_err(|e| format!("{}", e))
}

pub async fn start_kafka(pop_kafka: PopKafka) -> Result<ContainerAsync<Kafka>, String> {
    Kafka::default()
        .with_name(pop_kafka.image_name)
        .with_tag(pop_kafka.image_tag)
        .start()
        .await
        .map_err(|e| format!("{}", e))
}

pub async fn play_scenario(scenario_file: PopScenarioFile) -> Result<(), String> {
    let mut h_set = JoinSet::new();
    let mut children = Vec::new();

    info!("start setting up dependencies");

    let postgres_container: Option<ContainerAsync<Postgres>> =
        match &scenario_file.pop_test.pop_postgres {
            Some(postgres) => {
                info!("start setting up postgres");
                Some(start_postgres(postgres.clone()).await.unwrap())
            }
            _ => None,
        };

    let postgres_host: String = if let Some(p) = &postgres_container {
        p.get_host().await.unwrap().to_string()
    } else {
        "localhost".to_string()
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

    let kafka_container: Option<ContainerAsync<Kafka>> = match &scenario_file.pop_test.pop_kafka {
        Some(kafka) => {
            info!("start setting up kafka");
            Some(start_kafka(kafka.clone()).await.unwrap())
        }
        _ => None,
    };

    let kafka_host: String = if let Some(k) = &kafka_container {
        k.get_host().await.unwrap().to_string()
    } else {
        "localhost".to_string()
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

    let with_env = HashMap::from([
        (
            scenario_file
                .pop_test
                .pop_postgres
                .as_ref()
                .and_then(|pp| pp.db_var.clone())
                .unwrap_or("POSTGRES_DB".to_string()),
            "postgres".to_string(),
        ),
        (
            scenario_file
                .pop_test
                .pop_postgres
                .as_ref()
                .and_then(|pp| pp.user_var.clone())
                .unwrap_or("POSTGRES_USER".to_string()),
            "postgres".to_string(),
        ),
        (
            scenario_file
                .pop_test
                .pop_postgres
                .as_ref()
                .and_then(|pp| pp.pass_var.clone())
                .unwrap_or("POSTGRES_PASSWORD".to_string()),
            "postgres".to_string(),
        ),
        (
            scenario_file
                .pop_test
                .pop_postgres
                .as_ref()
                .and_then(|pp| pp.port_var.clone())
                .unwrap_or("POSTGRES_PORT".to_string()),
            postgres_port.to_string(),
        ),
        (
            scenario_file
                .pop_test
                .pop_postgres
                .as_ref()
                .and_then(|pp| pp.host_var.clone())
                .unwrap_or("POSTGRES_HOST".to_string()),
            postgres_host.clone(),
        ),
        (
            scenario_file
                .pop_test
                .pop_kafka
                .as_ref()
                .and_then(|pp| pp.bootstrap_server_var.clone())
                .unwrap_or("KAFKA_BOOTSTRAP_SERVER".to_string()),
            format!("{}:{}", kafka_host, kafka_port),
        ),
    ]);

    register_services(&mut h_set, &mut children, &scenario_file, with_env).await?;

    info!("awaiting for all services to be healthy");
    h_set.join_all().await;

    info!("performing tests and assertions");
    perform_checks(&scenario_file).await;

    terminate_processes(children).await;

    success!("finished playing scenario");
    Ok(())
}

pub async fn register_services(
    h_set: &mut JoinSet<()>,
    children: &mut Vec<Child>,
    scenario_file: &PopScenarioFile,
    with_env: HashMap<String, String>,
) -> Result<(), String> {
    for pop_service in scenario_file.pop_test.pop_services.clone() {
        info!("registering service: {}", pop_service.id);
        h_set.spawn(start_h(pop_service.clone()));
        let p = start_p(pop_service, with_env.clone())
            .await
            .map_err(|_| "Failed to start process")?;
        children.push(p);
    }
    Ok(())
}

pub async fn perform_checks(scenario_file: &PopScenarioFile) {
    let services = &scenario_file.pop_test.pop_services;

    for check in &scenario_file.pop_test.pop_checks {
        match check {
            PopCheck::Http {
                expected_code,
                check_url,
                service_id,
                json_body_patterns,
            } => {
                if let Some(service) = services.iter().find(|s| s.id == *service_id) {
                    info!("found service to check: {:?}", service);
                    info!("running check: {:?}", check);
                    if let Err(e) = do_with_http_check(
                        *expected_code,
                        check_url.clone(),
                        service.base_url.clone(),
                        json_body_patterns.clone(),
                    )
                    .await
                    {
                        error!("HTTP check failed: {:?}", e);
                    }
                } else {
                    error!("could not find service details for check");
                }
            }
        }
    }
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
