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

use tokio::time::sleep;

use crate::pop_model::{PopEnv, PostgresAssertion};
use crate::postgres::checker::postgres_checker_from_expression;
use crate::row_dsl::get_row_expression_quantifier;
use log::{error, info};
use std::time::Duration;
use tokio_postgres::NoTls;

pub mod checker;
pub mod container;

pub async fn do_postgres_assertion(
    pop_env: &PopEnv,
    postgres_assertion: &PostgresAssertion,
) -> Result<(), String> {
    if let Some(delay) = postgres_assertion.delay_seconds {
        sleep(Duration::from_secs(delay)).await;
    }

    postgres_assertion
        .name
        .clone()
        .inspect(|n| info!("starting action: {}", n));

    let (client, connection) = tokio_postgres::connect(
        pop_env
            .postgres_conn_string
            .clone()
            .unwrap_or("".to_string())
            .as_str(),
        NoTls,
    )
    .await
    .map_err(|e| e.to_string())?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });

    let rows = client
        .query(&postgres_assertion.postgres_query, &[])
        .await
        .map_err(|e| e.to_string())?;

    if let Some(rc) = postgres_assertion.assert_row_count {
        if rows.len() != (rc.to_owned() as usize) {
            error!("found an unexpected amount of rows in the table")
        } else {
            info!("found expected amount of rows in the table")
        }
    }

    for assert_expression in postgres_assertion
        .assert_expressions
        .clone()
        .unwrap_or_default()
    {
        let q = get_row_expression_quantifier(assert_expression.clone())?;
        info!("row expression quantifier: {:#?}", q);

        let row_checker = postgres_checker_from_expression(assert_expression.clone())?;

        match q {
            crate::row_dsl::RowExpressionQuantifier::All => {
                info!("TODO");
                for row in &rows {
                    let is_valid_row = row_checker(row);
                    info!("is valid row {}", is_valid_row);
                }
            }
            crate::row_dsl::RowExpressionQuantifier::First => {
                if !rows.is_empty() {
                    info!("reading first row");
                    let is_valid_row = row_checker(&rows[0]);
                    info!("is valid row {}", is_valid_row);
                } else {
                    error!("no rows were found for your query")
                }
            }
            crate::row_dsl::RowExpressionQuantifier::Last => {
                let pos = rows.len();
                info!("reading last row");
                let is_valid_row = row_checker(&rows[pos]);
                info!("is valid row {}", is_valid_row);
            }
            crate::row_dsl::RowExpressionQuantifier::Position(p) => {
                let pos = p as usize;
                info!("reading row at pos {}", pos);
                let is_valid_row = row_checker(&rows[pos]);
                info!("is valid row {}", is_valid_row);
            }
        };
    }

    Ok(())
}
