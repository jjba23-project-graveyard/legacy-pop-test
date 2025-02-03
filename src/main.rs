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

use pop_test::do_with_http_check;
use pop_test::start_h;
use pop_test::start_p;

use serde_yml;
use std::env;
use std::fs;
use std::process::exit;
use tokio::sync::watch;
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

    let xpr: Result<(), &'static str> = match maybe_file {
        Err(e) => {
            error!("error occurred reading file: {}", e);
            Err("err reading file")
        }
        Ok(file_data) => {
            info!("parsing YAML from file");
            let scenario_file: Result<pop_model::PopScenarioFile, serde_yml::Error> =
                serde_yml::from_str(&file_data);
            match scenario_file {
                Err(e) => {
                    error!("error occured parsing YAML {}", e);
                    Err("error parsing YAML")
                }
                Ok(s) => {
                    success!("parsed pop scenario properly from YAML file");
                    if user_cmd == "play-scenario" {
                        let mut h_set: JoinSet<_> = JoinSet::new();
                        let mut proc_set: JoinSet<_> = JoinSet::new();
                        let (tx, rx) = watch::channel(false);

                        for pop_service in s.pop_test.pop_services.clone() {
                            info!("registering service: {}", pop_service.id);
                            h_set.spawn(start_h(pop_service.clone()));
                            let p = start_p(pop_service.clone(), rx.clone());
                            proc_set.spawn(p);
                        }

                        info!("awaiting for all services to be healthy");
                        h_set.join_all().await;
                        info!("performing tests and assertions");

                        // here add checks

                        let servicios = s.pop_test.pop_services.clone();
                        for c in s.pop_test.pop_checks {
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
                                            let cc = do_with_http_check(
                                                expected_code,
                                                check_url,
                                                uu,
                                                body_patterns,
                                            )
                                            .await;

                                            match cc {
                                                Err(e) => error!("error occurred {}", e),
                                                _ => (),
                                            }
                                        }
                                        _ => error!("could not find service details for check"),
                                    }
                                }
                            }
                        }

                        info!("terminating all processes");
                        tx.send(true);
                        proc_set.shutdown().await;

                        success!("finished playing scenario");
                        Ok(())
                    } else {
                        let s = "could not understand the wanted task to execute";
                        Err(&s)
                    }
                }
            }
        }
    };
    xpr
}
