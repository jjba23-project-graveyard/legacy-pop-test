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
use std::env;
use std::fs;

mod pop_model;
mod pop_test;

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    info!("\n`pop-test` starting up\n");

    let args: Vec<String> = env::args().collect();
    info!("command-line arguments: {:?}", args);

    if args.len() != 3 {
        return Err("usage: <operation> <scenario-file>");
    }

    let (user_cmd, scenario_path) = (&args[1], &args[2]);
    info!("reading scenario file: {}", scenario_path);

    match fs::read_to_string(scenario_path) {
        Ok(file_data) => {
            success!("scenario file read successfully");
            pop_test::do_with_file_data(user_cmd.clone(), file_data).await
        }
        Err(e) => {
            error!("failed to read scenario file: {}", e);
            Err("error reading file")
        }
    }
}
