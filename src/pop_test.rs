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

use crate::pop_model::{BodyPattern, PopService, PopServiceHealthCheck};
use core::time::Duration;
use jql_runner::runner::raw;
use paris::{error, info, success};
use serde_json::json;
use std::{env, process::Stdio};
use tokio::{
    process::{Child, Command},
    time::sleep,
};

pub async fn do_with_http_check(
    expected_code: i32,
    check_url: String,
    service_url: String,
    body_patterns: Vec<BodyPattern>,
) -> Result<(), &'static str> {
    let a = raw(r#""a","b""#, &json!({ "a": 1, "b": 2 }));
    info!("{:?}", a);

    let url = format!("{}{}", service_url, check_url);

    let resp = reqwest::get(url.clone()).await;
    println!("{resp:#?}");

    match resp {
        Err(e) => {
            error!("{}", e);
        }
        Ok(v) => {
            let j = &v.json().await;
            match j {
                Ok(vv) => {
                    for pat in body_patterns {
                        let jj = raw(&pat.pattern, vv);
                        success!("{:?}", jj);
                    }
                    ()
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
) -> Result<(), &'static str> {
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
                Err("")
            }
        }
    }
}

pub async fn do_work(pop_service: PopService) -> Result<Child, std::io::Error> {
    info!("Task is running...");
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
    Command::new(bash_path)
        .args(&arg)
        .current_dir(dir)
        .envs(&pop_service.env_vars)
        .stdout(Stdio::inherit())
        .spawn()
}

pub async fn start_p(pop_service: PopService) -> Result<Child, std::io::Error> {
    do_work(pop_service).await
}

pub async fn start_h(pop_service: PopService) -> () {
    let pp = pop_service.clone();
    health_check(pp.id, pp.base_url, pp.health_check).await;
}
