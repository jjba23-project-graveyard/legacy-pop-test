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
use paris::success;
use serde_json::Value;

pub fn check_pat(body: &Value, patterns: Vec<BodyPattern>) -> Result<(), String> {
    for pat in patterns {
        let good_match = raw(&pat.pattern, body).map_err(|e| e.to_string())?;
        success!("{:?}", good_match);

        match pat.condition.as_str() {
            "!= \"\"" if good_match.as_str().unwrap_or("").is_empty() => {
                return Err("expectation not met of match to be non-empty".to_string());
            }
            cond if cond.starts_with("> ") => {
                let exp: f64 = str::replace(&pat.condition, "> ", "")
                    .parse()
                    .map_err(|e| format!("{}", e))?;

                let the_num: f64 = good_match.as_number().unwrap().as_f64().unwrap();
                if the_num <= exp {
                    return Err(format!(
                        "expectation not met of numeric value: {} {}",
                        the_num, &pat.condition
                    ));
                } else {
                    success!("{} {}", good_match, &pat.condition);
                    continue;
                }
            }
            _ => {}
        }

        success!("{} {}", good_match, pat.condition);
    }
    Ok(())
}

// module tests

#[cfg(test)]
mod tests {
    use core::assert_ne;

    use super::*;
    use paris::error;
    use serde_json::json;

    #[test]
    fn check_greater_than_pat_handles_happy_string() {
        let mut patterns = Vec::new();
        patterns.push(BodyPattern {
            pattern: "\"sut\"".to_string(),
            condition: "> 0".to_string(),
        });
        let body = json!({"sut": 1});
        let result = check_pat(&body, patterns);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn check_non_empty_string_pat_handles_happy_string() {
        let mut patterns = Vec::new();
        patterns.push(BodyPattern {
            pattern: "\"sut\"".to_string(),
            condition: "!= \"\"".to_string(),
        });
        let body = json!({"sut": "something"});
        let result = check_pat(&body, patterns);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn check_fails_gracefully_when_no_matches_found() {
        let mut patterns = Vec::new();
        patterns.push(BodyPattern {
            pattern: "\"suta\"".to_string(),
            condition: "!= \"\"".to_string(),
        });
        let body = json!({"sut": "something"});
        let result = check_pat(&body, patterns);
        error!("{:?}", result);
        assert_ne!(result, Ok(()));
    }
}
