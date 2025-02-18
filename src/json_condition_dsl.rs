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

fn translate_pattern(p: String) -> String {
    let x: Vec<String> = p.split(".").map(|s| s.to_string()).collect();
    info!("{:?}", x);
    let xx: Vec<String> = x
        .into_iter()
        .map(|fragment| {
            if fragment.starts_with("[") {
                fragment.to_string()
            } else {
                let pp = format!("\"{}\"", fragment);
                pp.clone()
            }
        })
        .collect();
    info!("{:?}", xx);

    xx.join("")
}

pub fn eval_json_condition(body: &Value, pattern: BodyPattern) -> Result<(), String> {
    info!("BEFORE TRANSLATE");
    let t_pattern = translate_pattern(pattern.pattern);
    info!("{:?}", t_pattern);

    let good_match = raw(&t_pattern, body).map_err(|e| e.to_string())?;

    match pattern.condition.as_str() {
        // match should not be empty
        "non-empty-str" => {
            if good_match.as_str() == Some("") {
                return Err("expectation not met of match to be non-empty".to_string());
            }
            info!("condition successful: {} {}", good_match, pattern.condition);
        }
        "empty-str" => {
            if good_match.as_str() != Some("") {
                return Err("expectation not met of match to be empty".to_string());
            }
            info!("condition successful: {} {}", good_match, pattern.condition);
        }
        // match should be different than
        cond if cond.starts_with("!= ") => match good_match {
            Value::String(ref s) => {
                if s == cond.strip_prefix("!= ").unwrap_or("".trim()) {
                    return Err(
                        "expectation not met of match to be different to value ".to_string()
                    );
                }
                info!("condition successful: {} {}", good_match, pattern.condition);
            }
            Value::Number(ref n) => {
                let nn: f64 = cond
                    .strip_prefix("!= ")
                    .unwrap_or("".trim())
                    .parse()
                    .map_err(|e| format!("{}", e))?;
                if n.as_f64() == Some(nn) {
                    return Err(
                        "the numbers should be different but were found to be equal".to_string()
                    );
                }
                info!("condition successful: {} {}", good_match, pattern.condition);
            }
            _ => return Err("could not apply condition checking to type".to_string()),
        },
        // match should be equal to
        cond if cond.starts_with("== ") => match good_match {
            Value::String(ref s) => {
                if s != cond.strip_prefix("== ").unwrap_or("".trim()) {
                    return Err("expectation not met of match to be equal to value ".to_string());
                }
                info!("condition successful: {} {}", good_match, pattern.condition);
            }
            Value::Number(ref n) => {
                let nn: f64 = cond
                    .strip_prefix("== ")
                    .unwrap_or("".trim())
                    .parse()
                    .map_err(|e| format!("{}", e))?;
                if n.as_f64() != Some(nn) {
                    return Err(
                        "the numbers should be equal but were found to be different".to_string()
                    );
                }
                info!("condition successful: {} {}", good_match, pattern.condition);
            }
            _ => return Err("could not apply condition checking to type".to_string()),
        },
        // match should be greater or smaller than x
        cond if cond.starts_with("> ") || cond.starts_with("< ") => {
            let exp: f64 = cond[2..].parse().map_err(|e| format!("{}", e))?;
            let the_num = good_match
                .as_number()
                .and_then(|n| n.as_f64())
                .ok_or_else(|| format!("expected numeric value, got: {:?}", good_match))?;

            let is_greater = pattern.condition.starts_with("> ");
            if (is_greater && the_num <= exp) || (!is_greater && the_num >= exp) {
                return Err(format!(
                    "expectation not met of numeric value: {} {}",
                    the_num, pattern.condition
                ));
            }
            info!("condition successful: {} {}", good_match, pattern.condition);
        }
        // match should be greater or equal or smaller or equal than x
        cond if cond.starts_with(">= ") || cond.starts_with("<= ") => {
            let exp: f64 = cond[3..].parse().map_err(|e| format!("{}", e))?;
            let the_num = good_match
                .as_number()
                .and_then(|n| n.as_f64())
                .ok_or_else(|| format!("expected numeric value, got: {:?}", good_match))?;

            let is_greater = pattern.condition.starts_with(">= ");
            if (is_greater && the_num < exp) || (!is_greater && the_num > exp) {
                return Err(format!(
                    "expectation not met of numeric value: {} {}",
                    the_num, pattern.condition
                ));
            }
            info!("condition successful: {} {}", good_match, pattern.condition);
        }
        // unknown conditions should result in error
        cond => {
            return Err(format!(
                "found a condition that was unable to process: {}",
                cond
            ))
        }
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

    use std::sync::Once;

    static INIT: Once = Once::new();

    pub fn initialize() {
        INIT.call_once(|| {
            env_logger::init();
        });
    }

    fn run_check_pat_test(
        pattern: &str,
        condition: &str,
        body: serde_json::Value,
        expected: Result<(), String>,
    ) {
        initialize();
        let result = eval_json_condition(
            &body,
            BodyPattern {
                pattern: pattern.to_string(),
                condition: condition.to_string(),
            },
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn check_greater_or_equal_than_pat_handles_happy_case() {
        run_check_pat_test("sut", ">= 0", json!({"sut": 0}), Ok(()));
    }

    #[test]
    fn check_greater_or_equal_than_pat_handles_happy_case2() {
        run_check_pat_test("sut", ">= 0", json!({"sut": 22}), Ok(()));
    }

    #[test]
    fn check_smaller_or_equal_than_pat_handles_happy_case() {
        run_check_pat_test("sut", "<= 0", json!({"sut": 0}), Ok(()));
    }

    #[test]
    fn check_smaller_or_equal_than_pat_handles_happy_case2() {
        run_check_pat_test("sut", "<= 0", json!({"sut": -22}), Ok(()));
    }

    #[test]
    fn check_smaller_than_pat_handles_happy_case() {
        run_check_pat_test("sut", "< 0", json!({"sut": -1}), Ok(()));
    }

    #[test]
    fn check_greater_than_pat_handles_happy_case() {
        run_check_pat_test("sut", "> 0", json!({"sut": 1}), Ok(()));
    }

    #[test]
    fn check_equals_pat_handles_happy_case() {
        run_check_pat_test("sut.sut2", "== 1", json!({"sut": {"sut2": 1}}), Ok(()));
    }

    #[test]
    fn check_not_equals_pat_pat_handles_happy_case() {
        run_check_pat_test("sut", "!= 0", json!({"sut": 1}), Ok(()));
    }

    #[test]
    fn check_non_empty_string_pat_handles_happy_case() {
        run_check_pat_test("sut", "non-empty-str", json!({"sut": "something"}), Ok(()));
    }

    #[test]
    fn check_empty_string_pat_handles_happy_case() {
        run_check_pat_test("sut", "empty-str", json!({"sut": ""}), Ok(()));
    }

    #[test]
    fn check_fails_gracefully_when_no_matches_found() {
        let body = json!({"sut": "something"});
        let result = eval_json_condition(
            &body,
            BodyPattern {
                pattern: "suta".to_string(),
                condition: "!= \"\"".to_string(),
            },
        );
        error!("{:?}", result);
        assert_ne!(result, Ok(()));
    }

    #[test]
    fn check_fails_gracefully_when_no_condition_matches_found() {
        let body = json!({"sut": "something"});
        let result = eval_json_condition(
            &body,
            BodyPattern {
                pattern: "sut".to_string(),
                condition: "weird-condition".to_string(),
            },
        );
        error!("{:?}", result);
        assert_ne!(result, Ok(()));
    }

    #[test]
    fn test_translate_pattern_basic() {
        assert_eq!(translate_pattern("data".to_string()), "\"data\"");
        assert_eq!(translate_pattern("meta".to_string()), "\"meta\"");
    }

    #[test]
    fn test_translate_pattern_with_index() {
        assert_eq!(translate_pattern("[0]".to_string()), "[0]");
        assert_eq!(translate_pattern("[123]".to_string()), "[123]");
    }

    #[test]
    fn test_translate_pattern_with_dot_notation() {
        assert_eq!(
            translate_pattern("data.[0].forecastDate".to_string()),
            "\"data\"[0]\"forecastDate\""
        );
        assert_eq!(
            translate_pattern("data.meta.green".to_string()),
            "\"data\"\"meta\"\"green\""
        );
    }

    #[test]
    fn test_translate_pattern_mixed() {
        assert_eq!(
            translate_pattern("user.details.[42].name".to_string()),
            "\"user\"\"details\"[42]\"name\""
        );
        assert_eq!(
            translate_pattern("settings.[3].theme.dark".to_string()),
            "\"settings\"[3]\"theme\"\"dark\""
        );
    }
}
