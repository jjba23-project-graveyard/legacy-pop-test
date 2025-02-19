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



use crate::row_dsl::extract_checker_clauses;
use log::info;
use rusqlite::Row;

pub fn sqlite_checker_from_expression(
    expression: String,
) -> Result<Box<dyn Fn(&Row) -> bool>, String> {
    let clauses = extract_checker_clauses(expression.as_str());
    info!("found checker clauses: {:#?}", clauses);

    let fun = move |ro: &Row| {
        let parsed_row: Vec<String> = Vec::new();
        let cs = clauses.iter().enumerate().clone();

        true
    };

    Ok(Box::new(fun))
}
