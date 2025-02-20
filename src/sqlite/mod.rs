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

use std::time::Duration;

use crate::{
    pop_model::{PopEnv, SqliteAction, SqliteAssertion},
    row_condition_dsl::eval_row_condition,
    row_dsl::{extract_checker_clauses, get_row_expression_quantifier},
};

use log::{debug, error, info};
use rusqlite::{Connection, Row};
use tokio::time::sleep;

pub fn sqlite_checker_from_expression(
    expression: String,
) -> Result<Box<dyn Fn(Vec<String>) -> bool>, String> {
    let clauses = extract_checker_clauses(expression.as_str());
    info!("found checker clauses: {:#?}", clauses);

    let fun = move |ro: Vec<String>| {
        let mut parsed_row: Vec<String> = Vec::new();
        let cs = clauses.iter().enumerate().clone();

        for (i, c) in cs.clone() {
            parsed_row.push(ro[i].clone());
        }

        info!("parsed row: {:#?}", parsed_row);

        let mut re: Vec<Result<(), String>> = Vec::new();
        for (i, c) in cs.clone() {
            if parsed_row.len() >= i {
                let eval_result = eval_row_condition(parsed_row[i].clone(), c.clone());
                re.push(eval_result);
            }
        }

        let all_re: Result<(), String> = re.into_iter().collect();
        all_re.is_ok()
    };

    Ok(Box::new(fun))
}

fn extract_row(row: &Row, col_count: usize) -> Vec<String> {
    (0..col_count)
        .map(|i| {
            let x: Result<Option<String>, String> = row
                .get::<usize, Option<String>>(i)
                .map_err(|e| e.to_string());
            match x {
                Ok(Some(val)) => val,
                Ok(None) => "nil".to_string(),
                Err(e) => {
                    error!("{}", e);
                    "ERROR".to_string()
                }
            }
        })
        .collect()
}

pub async fn do_sqlite_assertion(
    pop_env: &PopEnv,
    sqlite_assertion: &SqliteAssertion,
) -> Result<(), String> {
    if let Some(delay) = sqlite_assertion.delay_seconds {
        sleep(Duration::from_secs(delay)).await;
    }

    sqlite_assertion
        .name
        .clone()
        .inspect(|n| info!("starting action: {}", n));

    let conn = Connection::open(
        pop_env
            .sqlite_db_path
            .clone()
            .unwrap_or("./pop-test.sqlite".to_string()),
    )
    .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(&sqlite_assertion.sqlite_query)
        .map_err(|e| e.to_string())?;

    let col_co = stmt.column_count();
    let real_rc = stmt
        .query_map([], |ro| Ok(extract_row(ro, col_co)))
        .map_err(|e| e.to_string())?
        .count();

    if let Some(rc) = sqlite_assertion.assert_row_count {
        if real_rc != (rc.to_owned() as usize) {
            error!("found an unexpected amount of rows in the table")
        } else {
            info!("found expected amount of rows in the table")
        }
    }

    let mut xx = stmt
        .query_map([], |ro| Ok(extract_row(ro, col_co)))
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    while let Some(Ok(v)) = xx.next() {
        results.push(v);
    }

    info!("row results: {:?}", results);

    for assert_expression in sqlite_assertion
        .assert_expressions
        .clone()
        .unwrap_or_default()
    {
        let rc = results.clone();
        let q = get_row_expression_quantifier(assert_expression.clone())?;
        info!("row expression quantifier: {:#?}", q);

        info!("assert: {}", assert_expression);

        let row_checker = sqlite_checker_from_expression(assert_expression.clone())?;

        match q {
            crate::row_dsl::RowExpressionQuantifier::First => {
                if real_rc != 0_usize {
                    info!("reading first row");
                    let is_valid_row = row_checker(rc[0].clone());
                    info!("is valid row {}", is_valid_row);
                } else {
                    error!("no rows were found for your query")
                }
            }
            _ => info!(""),
        };
    }

    Ok(())
}

pub async fn do_sqlite_action(
    pop_env: &PopEnv,
    sqlite_action: &SqliteAction,
) -> Result<(), String> {
    if let Some(delay) = sqlite_action.delay_seconds {
        sleep(Duration::from_secs(delay)).await;
    }

    sqlite_action
        .name
        .clone()
        .inspect(|n| info!("starting action: {}", n));

    info!(
        "{}",
        pop_env
            .sqlite_db_path
            .clone()
            .unwrap_or("./pop-test.sqlite".to_string())
    );

    let conn = Connection::open(
        pop_env
            .sqlite_db_path
            .clone()
            .unwrap_or("./pop-test.sqlite".to_string()),
    )
    .map_err(|e| e.to_string())?;

    let qs: Vec<String> = sqlite_action
        .sqlite_action
        .split(";")
        .map(|s| {
            let ss = s.replace("\n", " ");
            ss.clone().trim().to_string()
        })
        .filter(|s| !s.is_empty())
        .collect();

    debug!("query vec: {:?}", qs);

    for q in qs {
        debug!("query: {}", q);
        let r = conn.execute(q.as_str(), []).map_err(|e| e.to_string())?;
        debug!("result: {}", r);
    }

    Ok(())
}
