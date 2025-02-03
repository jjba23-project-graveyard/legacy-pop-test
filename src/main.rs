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
use pop_test::do_with_http_check;
use pop_test::start_h;
use pop_test::start_p;

use serde_yml;
use std::env;
use std::fs;
use tokio::process::Child;
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
        let msg =
            "did not pass the correct command-line arguments, expected <operation> <scenario-file>";
        error!("{}", msg);
        return Err(msg);
    }

    let user_cmd = &args[1];
    let scenario_path = &args[2];

    info!("reading scenario file");
    let maybe_file = fs::read_to_string(scenario_path);

    match maybe_file {
        Err(e) => {
            error!("error occurred reading file: {}", e);
            Err("err reading file")
        }
        Ok(file_data) => do_with_file_data(user_cmd.to_string(), file_data).await,
    }
}

async fn play_scenario(scenario_file: PopScenarioFile) -> Result<(), &'static str> {
    let mut h_set: JoinSet<_> = JoinSet::new();
    let mut children: Vec<Child> = Vec::new();

    for pop_service in scenario_file.pop_test.pop_services.clone() {
        info!("registering service: {}", pop_service.id);
        h_set.spawn(start_h(pop_service.clone()));
        let p = start_p(pop_service.clone()).await.unwrap();
        children.push(p);
    }

    info!("awaiting for all services to be healthy");
    h_set.join_all().await;
    info!("performing tests and assertions");

    // here add checks

    let servicios = scenario_file.pop_test.pop_services.clone();
    for c in scenario_file.pop_test.pop_checks {
        let cccc = c.clone();
        match c {
            pop_model::PopCheck::Http {
                expected_code,
                check_url,
                service_id,
                body_patterns,
            } => {
                let ss = service_id.clone();
                let maybe_service = servicios.iter().find(|x| x.id == ss);

                match maybe_service {
                    Some(s) => {
                        info!("found service to check : {:?}", s);
                        info!("running check: {:?}", cccc);
                        let uu = s.base_url.clone();

                        do_with_http_check(expected_code, check_url, uu, body_patterns)
                            .await
                            .unwrap()
                    }
                    _ => error!("could not find service details for check"),
                }
            }
        }
    }

    info!("terminating all processes");

    for mut child in children {
        info!("terminating pid: {:?}", child.id());
        let _ = child.kill().await;
    }

    success!("finished playing scenario");
    Ok(())
}

async fn do_with_file_data(user_cmd: String, file_data: String) -> Result<(), &'static str> {
    info!("parsing YAML from file");
    let scenario_file: pop_model::PopScenarioFile = serde_yml::from_str(&file_data).unwrap();

    success!("parsed pop scenario properly from YAML file");
    if user_cmd == "play-scenario" {
        play_scenario(scenario_file).await
    } else {
        let msg = "could not understand the wanted task to execute";
        error!("{}", msg);
        Err(&msg)
    }
}
