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

use json_schema::generate_json_schema;
use log::{error, info};
use pop_model::PopScenario;
use scenario::play_scenario;
use std::{fs, path::Path};

mod action;
mod containers;
mod environment;
mod health_check;
mod json_condition_dsl;
mod pop_model;
mod proc;
mod row_condition_dsl;
mod row_dsl;
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

#[path = "json_schema/mod.rs"]
mod json_schema;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short('p'), long("path"))]
    path: Option<String>,
    #[arg(short('g'), long("generate-json-schema"))]
    generate_json_schema: Option<bool>,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    env_logger::init();
    info!("\npop-test starting up\n");

    info!("parsing command-line arguments");
    let args = Args::parse();

    if let Some(path) = args.path {
        info!("gathering scenario files in desired path");
        let file_paths = gather_file_paths(path.clone());
        info!("gathered files: {:#?}", file_paths);

        async {
            for f in &file_paths {
                info!("scenario file read successfully to string");

                do_with_file_data(
                    f.split(".").last().unwrap_or("yaml").to_string(),
                    f.to_string(),
                )
                .await
                .map_err(|e| {
                    error!("{}", e);
                    "scenario failed to run"
                })?;

                info!("✅ ran scenario successfully");
            }
            info!("done running {} scenarios", &file_paths.len());
            Ok(())
        }
        .await
    } else if args.generate_json_schema.is_some() {
        let r = generate_json_schema().await;
        if r.is_ok() {
            info!("successfully generated JSON schema for pop-scenario")
        };
        r
    } else {
        async { Err("could not start pop-test since CLI arguments were invalid".to_string()) }.await
    }
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
                .inspect(|_| info!("parsed YAML successfully"))
                .ok()
        }
        "toml" => {
            info!("parsing TOML from file");
            toml::from_str(&file_data)
                .inspect_err(|e| error!("{}", e))
                .inspect(|_| info!("parsed TOML successfully"))
                .ok()
        }
        "json" => {
            info!("parsing JSON from file");
            serde_json::from_str(&file_data)
                .inspect_err(|e| error!("{}", e))
                .inspect(|_| info!("parsed JSON successfully"))
                .ok()
        }
        _ => None,
    };

    match maybe_scenario {
        Some(scenario) => play_scenario(scenario).await,
        None => Err("could not find a valid scenario to play".to_string()),
    }
}

pub fn gather_file_paths(path: String) -> Vec<String> {
    let p = Path::new(&path);

    if p.is_dir() {
        fs::read_dir(p)
            .unwrap()
            .filter_map(Result::ok)
            .flat_map(|entry| gather_file_paths(entry.path().to_string_lossy().to_string()))
            .collect()
    } else {
        vec![path.to_string()]
    }
}
