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

use core::fmt;
use http_body_util::Empty;
use paris::{error, info, success, warn};
use std::{
    env,
    process::{Command, Stdio},
};
use tokio::net::TcpStream;

use crate::pop_model::{PopScenario, PopService};

pub async fn health_check(pop_service: PopService) -> Result<(), &'static str> {
    info!("starting healthcheck for service: {}", pop_service.id);
    match service.health_check {
        crate::pop_model::PopServiceHealthCheck::Http {
            path,
            patience_seconds,
        } => {
            let resp = reqwest::get(format!("{}{}", pop_service.base_url, path))
                .await?
                .json::<HashMap<String, String>>()
                .await?;
            println!("{resp:#?}");
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

pub async fn play_scenario(scenario: PopScenario) -> Result<(), &'static str> {
    for pop_service in scenario.pop_services {
        info!("registering service: {}", pop_service.id);
        let user_home = env::var("HOME").unwrap();
        let bash_path = str::replace(&pop_service.bash_path, "$HOME", &user_home);

        let dir = str::replace(&pop_service.dir, "$HOME", &user_home);

        info!("in dir {}", &dir);
        info!("with args {:?}", &pop_service.start_cmd_args);

        let mut arg: Vec<String> = Vec::new();
        arg.push("-c".to_string());
        for b in pop_service.start_cmd_args {
            arg.push(b);
        }

        info!("bash with args {:?}", &arg);

        let proc: tokio::task::JoinHandle<_> = tokio::spawn(async move {
            let _ = Command::new(bash_path)
                .args(&arg)
                .current_dir(dir)
                .envs(&pop_service.env_vars)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .output()
                .expect("failed to execute process");
        });

        let h: tokio::task::JoinHandle<_> = health_check(pop_service);

        match proc.await {
            Err(e) => {
                error!("error occurred while executing proc: {}", e)
            }
            Ok(_) => {
                info!("success running proc")
            }
        }
    }

    Ok(())
}
