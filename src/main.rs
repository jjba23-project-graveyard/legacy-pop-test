use serde::{Deserialize, Serialize};
use serde_yml;
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
enum PopServiceHealthCheck {
    Http { path: String, patience_seconds: i32 },
    Placebo,
}

#[derive(Serialize, Deserialize, Debug)]
struct PopService {
    id: String,
    name: String,
    dir: String,
    base_url: String,
    start_cmd: String,
    env_vars: HashMap<String, String>,
    health_check: PopServiceHealthCheck,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct BodyPattern {
    pattern: String,
    condition: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
enum PopCheck {
    Http {
        expected_code: i32,
        check_url: String,
        service_id: String,
        body_patterns: Vec<BodyPattern>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct PopScenario {
    pop_services: Vec<PopService>,
    pop_checks: Vec<PopCheck>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PopScenarioFile {
    poppenspeler: PopScenario,
}

fn main() -> Result<(), serde_yml::Error> {
    let p = "./resources/scenarios/test-green-energy-mix.yaml";
    let f = fs::read_to_string(p);
    match f {
        Err(e) => println!("error occurred reading file: {}", e),
        Ok(file_data) => {
            let scenario_file: PopScenarioFile = serde_yml::from_str(&file_data)?;
            println!("parsed PopScenarioFile:");
            println!("{:?}", scenario_file);
        }
    }

    Ok(())
}
