// poppenspeler - your test orchestration master of puppets

// Copyright (C) 2025 Josep Bigorra (jjba23) <jjbigorra@gmail.com>

// poppenspeler is free software: you can redistribute it and/or modify
// it under the terms of the Lesser GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// poppenspeler is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// Lesser GNU General Public License for more details.

// You should have received a copy of the Lesser GNU General Public License
// along with poppenspeler.  If not, see <https://www.gnu.org/licenses/>.

use core::fmt;
use paris::{error, info, success, warn};
use std::{
    env,
    process::{Command, Stdio},
};

use crate::pop_model::PopScenario;

pub fn play_scenario(scenario: PopScenario) -> () {
    info!("{:?}", scenario);

    for pop_service in &scenario.pop_services {
        let user_home = env::var("HOME").unwrap();
        let bash_path = str::replace(&pop_service.bash_path, "$HOME", &user_home);

        let dir = str::replace(&pop_service.dir, "$HOME", &user_home);

        info!("in dir {}", &dir);
        info!("with args {:?}", &pop_service.start_cmd_args);

        let mut arg = Vec::new();
        arg.push("-c");
        for b in &pop_service.start_cmd_args {
            arg.push(b);
        }

        info!("bash with args {:?}", &arg);

        let _ = Command::new(bash_path)
            .args(&arg)
            .current_dir(dir)
            .envs(&pop_service.env_vars)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .output()
            .expect("failed to execute process");
    }
}
