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

use tokio_postgres::Row;
use uuid::Uuid;

use std::time::SystemTime;

use crate::{row_condition_dsl::eval_row_condition, row_dsl::extract_checker_clauses};
use chrono::{DateTime, Utc};
use log::{debug, error, info};

pub fn postgres_checker_from_expression(
    expression: String,
) -> Result<Box<dyn Fn(&Row) -> bool>, String> {
    let clauses = extract_checker_clauses(expression.as_str());
    info!("found checker clauses: {:#?}", clauses);

    let fun = move |ro: &Row| {
        let mut parsed_row: Vec<String> = Vec::new();
        let cs = clauses.iter().enumerate().clone();
        for (i, clause) in cs.clone() {
            let col_type = ro.columns()[i].type_().name();

            debug!("working with column type: {}", col_type);
            match col_type {
                "uuid" => {
                    let uid: Result<Option<Uuid>, String> =
                        ro.try_get(i).map_err(|e| e.to_string());
                    match uid {
                        Ok(Some(uuid)) => {
                            let mut u = Uuid::encode_buffer();
                            let aa = uuid.hyphenated().encode_lower(&mut u);
                            parsed_row.push(aa.to_owned());
                            debug!("{}", aa.to_owned());
                        }
                        Ok(None) => parsed_row.push("nil".to_string()),
                        Err(e) => {
                            error!("{}", e);
                            parsed_row.push("ERROR".to_string())
                        }
                    }
                }
                "varchar" => {
                    let maybe_str: Result<Option<String>, String> =
                        ro.try_get(i).map_err(|e| e.to_string());
                    match maybe_str {
                        Ok(Some(val)) => {
                            parsed_row.push(val.to_owned());
                        }
                        Ok(None) => parsed_row.push("nil".to_string()),
                        Err(e) => error!("{}", e),
                    }
                }
                "int4" => {
                    let maybe_i: Result<Option<i32>, String> =
                        ro.try_get(i).map_err(|e| e.to_string());
                    match maybe_i {
                        Ok(Some(val)) => {
                            parsed_row.push(val.to_string().to_owned());
                        }
                        Ok(None) => parsed_row.push("nil".to_string()),
                        Err(e) => {
                            error!("{}", e);
                            parsed_row.push("ERROR".to_string())
                        }
                    }
                }
                "bool" => {
                    let maybe_bool: Result<Option<bool>, String> =
                        ro.try_get(i).map_err(|e| e.to_string());
                    match maybe_bool {
                        Ok(Some(val)) => {
                            parsed_row.push(val.to_string().to_owned());
                        }
                        Ok(None) => parsed_row.push("nil".to_string()),
                        Err(e) => {
                            error!("{}", e);
                            parsed_row.push("ERROR".to_string())
                        }
                    }
                }
                "timestamp" => {
                    let ts: Result<Option<SystemTime>, String> =
                        ro.try_get(i).map_err(|e| e.to_string());
                    match ts {
                        Ok(Some(st)) => {
                            let dt: DateTime<Utc> = st.into();
                            parsed_row.push(dt.to_rfc3339());
                        }
                        Ok(None) => parsed_row.push("nil".to_string()),
                        Err(e) => {
                            error!("{}", e);
                            parsed_row.push("ERROR".to_string())
                        }
                    }
                }
                _ => {
                    debug!("other column type: {}", col_type);
                    let x: Result<Option<String>, String> =
                        ro.try_get(i).map_err(|e| e.to_string());
                    match x {
                        Ok(Some(got)) => {
                            debug!("test on other: {:?}", got);
                            parsed_row.push(got.to_owned());
                        }
                        Ok(None) => parsed_row.push("nil".to_string()),
                        Err(e) => {
                            error!("{}", e);
                            parsed_row.push("ERROR".to_string())
                        }
                    }
                }
            }
        }

        info!("parsed row: {:#?}", parsed_row);

        let mut re: Vec<Result<(), String>> = Vec::new();
        for (i, clause) in cs.clone() {
            if ro.columns().len() >= i && parsed_row.len() >= i {
                let eval_result = eval_row_condition(parsed_row[i].clone(), clause.clone());
                re.push(eval_result);
            }
        }

        let all_re: Result<(), String> = re.into_iter().collect();
        all_re.is_ok()
    };

    Ok(Box::new(fun))
}
