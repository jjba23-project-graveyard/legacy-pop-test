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

use log::info;
use std::collections::HashMap;
use std::{env, process::Stdio};
use tokio::process::{Child, Command};

use crate::pop_model::PopService;

pub async fn terminate_services(children: Vec<Child>) {
    info!("terminating all processes");
    for mut child in children {
        info!("terminating pid: {:?}", child.id());
        let _ = child.kill().await;
    }
}

pub async fn start_service(
    pop_service: PopService,
    with_env: HashMap<String, String>,
) -> Result<Child, std::io::Error> {
    run_proc_from_service(pop_service, with_env).await
}

async fn run_proc_from_service(
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
