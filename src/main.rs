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

use pop_test::start_h;
use pop_test::start_p;
use serde_yml;
use std::env;
use std::fs;
use std::process::exit;
use tokio::task::JoinSet;

mod pop_model;
mod pop_test;

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    println!("");
    info!("`pop-test` starting up");
    println!("");

    info!("reading command-line arguments");

    let args: Vec<String> = env::args().collect();
    info!("{:?}", args);

    if args.len() != 3 {
        error!(
            "did not pass the correct command-line arguments, expected <operation> <scenario-file>"
        );
        exit(1);
    }

    let user_cmd = &args[1];
    let scenario_path = &args[2];

    info!("reading scenario file");
    let maybe_file = fs::read_to_string(scenario_path);

    let _ = match maybe_file {
        Err(e) => {
            error!("error occurred reading file: {}", e);
        }
        Ok(file_data) => {
            info!("parsing YAML from file");
            let scenario_file: Result<pop_model::PopScenarioFile, serde_yml::Error> =
                serde_yml::from_str(&file_data);
            match scenario_file {
                Err(e) => {
                    error!("error occured parsing YAML {}", e);
                }
                Ok(s) => {
                    success!("parsed pop scenario properly from YAML file");
                    if user_cmd == "play-scenario" {
                        //tokio::join!(play_scenario(s.pop_test));
                        for pop_service in s.pop_test.pop_services {
                            info!("registering service: {}", pop_service.id);
                            let mut set: JoinSet<_> = JoinSet::new();
                            set.spawn(start_h(pop_service.clone()));

                            let mut proc_set: JoinSet<_> = JoinSet::new();
                            proc_set.spawn(async move { start_p(pop_service.clone()) });

                            set.join_all().await;
                            proc_set.join_all().await;
                        }
                        success!("finished playing scenario");
                    }
                }
            }
        }
    };

    Ok(())
}
