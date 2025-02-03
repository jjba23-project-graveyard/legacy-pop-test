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

use paris::{error, info};
use std::env;
use std::fs;

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
        Ok(file_data) => pop_test::do_with_file_data(user_cmd.to_string(), file_data).await,
    }
}
