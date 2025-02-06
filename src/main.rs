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

use paris::{error, info, success};
use pop_model::PopScenarioFile;
use scenario::play_scenario;
use std::env;
use std::fs;

mod checks;
mod condition_dsl;
mod health_check;
mod kafka;
mod pop_model;
mod postgres;
mod proc;
mod scenario;

#[tokio::main]
async fn main() -> Result<(), String> {
    info!("\n`pop-test` starting up\n");

    let args: Vec<String> = env::args().collect();
    info!("command-line arguments: {:?}", args);

    if args.len() != 3 {
        return Err("usage: <operation> <scenario-file>".to_string());
    }

    let (user_cmd, scenario_path) = (&args[1], &args[2]);
    info!("reading scenario file: {}", scenario_path);

    let file_data: String = fs::read_to_string(scenario_path).map_err(|e| {
        let err_msg = format!("failed to read scenario file: {}", e);
        error!("{}", err_msg);
        err_msg
    })?;

    async {
        success!("scenario file read successfully to string");

        do_with_file_data(user_cmd.clone(), file_data)
            .await
            .map_err(|e| {
                error!("{}", e);
                "scenario failed to run successfully"
            })?;

        success!("ran scenario successfully");
        Ok(())
    }
    .await
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
