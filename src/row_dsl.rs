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

use std::time::SystemTime;

use chrono::{DateTime, Utc};
use log::{debug, error, info};
// You should have received a copy of the Lesser GNU General Public License
// along with pop-test.  If not, see <https://www.gnu.org/licenses/>.
use regex::Regex;
use tokio_postgres::Row;
use uuid::Uuid;

use crate::row_condition_dsl::eval_row_condition;

#[derive(PartialEq, Debug, Clone)]
pub enum RowExpressionQuantifier {
    All,
    First,
    Last,
    Position(i32),
}

fn extract_checker_clauses(input: &str) -> Vec<String> {
    let mut depth = 0;
    let mut clause = String::new();

    input
        .chars()
        .filter_map(|c| match c {
            '(' => {
                depth += 1;
                None
            }
            ')' => {
                depth -= 1;
                if !clause.is_empty() && depth >= 1 {
                    let extracted = Some(clause.trim().to_string());
                    clause.clear();
                    extracted
                } else {
                    None
                }
            }
            _ if depth >= 1 => {
                clause.push(c);
                None
            }
            _ => None,
        })
        .collect()
}

pub fn make_checker_from_expression(
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

        info!("{:#?}", parsed_row);

        for (i, clause) in cs.clone() {
            if ro.columns().len() >= i && parsed_row.len() >= i {
                let col_type = ro.columns()[i].type_().name();
                let eval_result =
                    eval_row_condition(parsed_row[i].clone(), clause.clone(), col_type.to_string());
                info!("{:#?}", eval_result);
            }
        }

        true
    };

    Ok(Box::new(fun))
}

pub fn get_row_expression_quantifier(
    expression: String,
) -> Result<RowExpressionQuantifier, String> {
    if expression.starts_with("[all]") {
        return Ok(RowExpressionQuantifier::All);
    }

    if expression.starts_with("[first]") || expression.starts_with("[0]") {
        return Ok(RowExpressionQuantifier::First);
    }

    if expression.starts_with("[last]") || expression.starts_with("[-1]") {
        return Ok(RowExpressionQuantifier::Last);
    }

    let re = Regex::new(r"(\[.+\])").unwrap();

    match re.captures(expression.as_str()) {
        Some(captures) => {
            let full_match = captures.get(1).unwrap().as_str();
            info!("Extracted key: {}", full_match);

            let num_re = Regex::new(r"-?\d+").unwrap();
            if let Some(num_match) = num_re.find(full_match) {
                let number = num_match.as_str();
                info!("Extracted number: {}", number);
                let num = number.parse::<i32>().map_err(|e| e.to_string())?;
                if num < 0 {
                    let msg: String = "a row expression quantifier cannot be negative".to_owned();
                    return Err(msg);
                }
                return Ok(RowExpressionQuantifier::Position(num));
            }

            let msg: String =
                "no matches were found that matched row expression quantifiers".to_owned();
            Err(msg)
        }
        _ => {
            let msg: String =
                "no matches were found that matched row expression quantifiers".to_owned();
            Err(msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_row_expression_quantifier_all() {
        let result = get_row_expression_quantifier("[all]".to_string());
        assert_eq!(result, Ok(RowExpressionQuantifier::All));
    }

    #[test]
    fn test_get_row_expression_quantifier_first() {
        let result = get_row_expression_quantifier("[first]".to_string());
        assert_eq!(result, Ok(RowExpressionQuantifier::First));
    }

    #[test]
    fn test_get_row_expression_quantifier_last_min_one() {
        let result = get_row_expression_quantifier("[-1]".to_string());
        assert_eq!(result, Ok(RowExpressionQuantifier::Last));
    }

    #[test]
    fn test_get_row_expression_quantifier_last() {
        let result = get_row_expression_quantifier("[last]".to_string());
        assert_eq!(result, Ok(RowExpressionQuantifier::Last));
    }

    #[test]
    fn test_get_row_expression_quantifier_position_valid() {
        let result = get_row_expression_quantifier("[42]".to_string());
        assert_eq!(result, Ok(RowExpressionQuantifier::Position(42)));
    }

    #[test]
    fn test_get_row_expression_quantifier_position_negative() {
        let result = get_row_expression_quantifier("[-10]".to_string());
        assert!(result.is_err(), "Expected an error for negative numbers.");
    }

    #[test]
    fn test_get_row_expression_quantifier_invalid_format() {
        let result = get_row_expression_quantifier("[notanumber]".to_string());
        assert!(result.is_err(), "Expected an error for invalid format");
    }

    #[test]
    fn test_get_row_expression_quantifier_empty_string() {
        let result = get_row_expression_quantifier("".to_string());
        assert!(result.is_err(), "Expected an error for empty input.");
    }

    #[test]
    fn test_get_row_expression_quantifier_no_brackets() {
        let result = get_row_expression_quantifier("random text".to_string());
        assert!(result.is_err(), "Expected an error for missing brackets.");
    }

    #[test]
    fn test_get_row_expression_quantifier_mixed_brackets() {
        let result = get_row_expression_quantifier("[123abc]".to_string());
        assert_eq!(result, Ok(RowExpressionQuantifier::Position(123)));
    }

    // clause extraction tests

    #[test]
    fn test_basic_extraction() {
        let input = "[0] ((non-empty-str)(== 232323)(== en_GB)(non-empty-str))";
        let expected = vec![
            "non-empty-str".to_string(),
            "== 232323".to_string(),
            "== en_GB".to_string(),
            "non-empty-str".to_string(),
        ];
        assert_eq!(extract_checker_clauses(input), expected);
    }

    #[test]
    fn test_nested_parentheses() {
        let input = "[0] (((== nested)))";
        let expected = vec!["== nested".to_string()];
        assert_eq!(extract_checker_clauses(input), expected);
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let expected: Vec<String> = vec![];
        assert_eq!(extract_checker_clauses(input), expected);
    }

    #[test]
    fn test_no_parentheses() {
        let input = "[0] == value";
        let expected: Vec<String> = vec![];
        assert_eq!(extract_checker_clauses(input), expected);
    }

    #[test]
    fn test_malformed_input() {
        let input = "[0] ((== missing end";
        let expected: Vec<String> = vec![];
        assert_eq!(extract_checker_clauses(input), expected);
    }
}
