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

use crate::pop_model::BodyPattern;

use jql_runner::runner::raw;
use log::info;
use serde_json::Value;

pub fn eval_condition(body: &Value, patterns: Vec<BodyPattern>) -> Result<(), String> {
    for pat in patterns {
        let good_match = raw(&pat.pattern, body).map_err(|e| e.to_string())?;
        info!("{:?}", good_match);

        match pat.condition.as_str() {
            // match should not be empty
            "!= \"\"" => {
                if good_match.as_str().unwrap_or("").is_empty() {
                    return Err("expectation not met of match to be non-empty".to_string());
                }
            }
            "== \"\"" => {
                if !good_match.as_str().unwrap_or("").is_empty() {
                    return Err("expectation not met of match to be empty".to_string());
                }
            }
            // match should be greater or smaller than x
            cond if cond.starts_with("> ") || cond.starts_with("< ") => {
                let exp: f64 = cond[2..].parse().map_err(|e| format!("{}", e))?;
                let the_num = good_match
                    .as_number()
                    .and_then(|n| n.as_f64())
                    .ok_or_else(|| format!("expected numeric value, got: {:?}", good_match))?;

                let is_greater = pat.condition.starts_with("> ");
                if (is_greater && the_num <= exp) || (!is_greater && the_num >= exp) {
                    return Err(format!(
                        "expectation not met of numeric value: {} {}",
                        the_num, pat.condition
                    ));
                }
            }
            // match should be greater or equal or smaller or equal than x
            cond if cond.starts_with(">= ") || cond.starts_with("<= ") => {
                let exp: f64 = cond[3..].parse().map_err(|e| format!("{}", e))?;
                let the_num = good_match
                    .as_number()
                    .and_then(|n| n.as_f64())
                    .ok_or_else(|| format!("expected numeric value, got: {:?}", good_match))?;

                let is_greater = pat.condition.starts_with(">= ");
                if (is_greater && the_num < exp) || (!is_greater && the_num > exp) {
                    return Err(format!(
                        "expectation not met of numeric value: {} {}",
                        the_num, pat.condition
                    ));
                }
            }
            // unknown conditions should result in error
            cond => {
                return Err(format!(
                    "found a condition that was unable to process: {}",
                    cond
                ))
            }
        }

        info!("{} {}", good_match, pat.condition);
    }
    Ok(())
}

// module tests

#[cfg(test)]
mod tests {
    use core::assert_ne;

    use super::*;
    use log::error;
    use serde_json::json;

    fn run_check_pat_test(
        pattern: &str,
        condition: &str,
        body: serde_json::Value,
        expected: Result<(), String>,
    ) {
        let patterns = vec![BodyPattern {
            pattern: pattern.to_string(),
            condition: condition.to_string(),
        }];
        let result = eval_condition(&body, patterns);
        assert_eq!(result, expected);
    }

    #[test]
    fn check_greater_or_equal_than_pat_handles_happy_case() {
        run_check_pat_test("\"sut\"", ">= 0", json!({"sut": 0}), Ok(()));
    }

    #[test]
    fn check_greater_or_equal_than_pat_handles_happy_case2() {
        run_check_pat_test("\"sut\"", ">= 0", json!({"sut": 22}), Ok(()));
    }

    #[test]
    fn check_smaller_or_equal_than_pat_handles_happy_case() {
        run_check_pat_test("\"sut\"", "<= 0", json!({"sut": 0}), Ok(()));
    }

    #[test]
    fn check_smaller_or_equal_than_pat_handles_happy_case2() {
        run_check_pat_test("\"sut\"", "<= 0", json!({"sut": -22}), Ok(()));
    }

    #[test]
    fn check_smaller_than_pat_handles_happy_case() {
        run_check_pat_test("\"sut\"", "< 0", json!({"sut": -1}), Ok(()));
    }

    #[test]
    fn check_greater_than_pat_handles_happy_case() {
        run_check_pat_test("\"sut\"", "> 0", json!({"sut": 1}), Ok(()));
    }

    #[test]
    fn check_non_empty_string_pat_handles_happy_case() {
        run_check_pat_test("\"sut\"", "!= \"\"", json!({"sut": "something"}), Ok(()));
    }

    #[test]
    fn check_empty_string_pat_handles_happy_case() {
        run_check_pat_test("\"sut\"", "== \"\"", json!({"sut": ""}), Ok(()));
    }

    #[test]
    fn check_fails_gracefully_when_no_matches_found() {
        let patterns = vec![BodyPattern {
            pattern: "\"suta\"".to_string(),
            condition: "!= \"\"".to_string(),
        }];
        let body = json!({"sut": "something"});
        let result = eval_condition(&body, patterns);
        error!("{:?}", result);
        assert_ne!(result, Ok(()));
    }

    #[test]
    fn check_fails_gracefully_when_no_condition_matches_found() {
        let patterns = vec![BodyPattern {
            pattern: "\"sut\"".to_string(),
            condition: "weird-condition".to_string(),
        }];
        let body = json!({"sut": "something"});
        let result = eval_condition(&body, patterns);
        error!("{:?}", result);
        assert_ne!(result, Ok(()));
    }
}
