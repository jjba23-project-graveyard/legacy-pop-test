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

use log::{error, info};
use pop_model::PopScenario;
use scenario::play_scenario;
use std::{fs, path::Path};

mod condition_dsl;
mod containers;
mod environment;
mod health_check;
mod pop_model;
mod proc;
mod scenario;

#[path = "http/mod.rs"]
mod http;

#[path = "kafka/mod.rs"]
mod kafka;

#[path = "keycloak/mod.rs"]
mod keycloak;

#[path = "pop_server/mod.rs"]
mod pop_server;

#[path = "postgres/mod.rs"]
mod postgres;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short('f'), long("path"))]
    path: String,

    #[arg(short('t'), long("type"))]
    file_type: String,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    env_logger::init();
    info!("\n`pop-test` starting up\n");

    info!("parsing command-line arguments");
    let args = Args::parse();

    info!("gathering scenario files in desired path");
    let p = Path::new(&args.path);

    let mut file_paths = Vec::new();

    if p.is_dir() {
        fs::read_dir(args.path)
            .unwrap()
            .for_each(|fp| file_paths.push(fp.unwrap().path().to_string_lossy().to_string()));
    } else {
        file_paths.push(p.to_string_lossy().to_string())
    }

    async {
        for f in file_paths {
            info!("scenario file read successfully to string");

            do_with_file_data(args.file_type.clone(), f.to_string())
                .await
                .map_err(|e| {
                    error!("{}", e);
                    "scenario failed to run"
                })?;

            info!("✅ ran scenario successfully");
        }
        Ok(())
    }
    .await
}

pub async fn do_with_file_data(file_type: String, scenario_path: String) -> Result<(), String> {
    info!("reading file contents at: {}", scenario_path);

    let file_data: String = fs::read_to_string(scenario_path).map_err(|e| {
        let err_msg = format!("failed to read scenario file: {}", e);
        error!("{}", err_msg);
        err_msg
    })?;

    let maybe_scenario: Option<PopScenario> = match file_type.as_str() {
        "yaml" | "yml" => {
            info!("parsing YAML from file");
            serde_yml::from_str(&file_data)
                .inspect_err(|e| error!("{}", e))
                .ok()
        }
        _ => None,
    };

    match maybe_scenario {
        Some(scenario) => play_scenario(scenario).await,
        None => Err("could not find a valid scenario to play".to_string()),
    }
}
