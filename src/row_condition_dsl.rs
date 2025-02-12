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

pub fn eval_row_condition(
    target: String,
    condition: String,
    col_type: String,
) -> Result<(), String> {
    match condition.as_str() {
        // match should not be empty
        "non-empty-str" => {
            if target.is_empty() {
                return Err("expectation not met of match to be non-empty".to_string());
            }
        }
        "empty-str" => {
            if !target.is_empty() {
                return Err("expectation not met of match to be empty".to_string());
            }
        }
        // match should be different than
        cond if cond.starts_with("!= ") => {
            // TODO handle col_type here, add int support
            if target == cond.strip_prefix("!= ").unwrap_or("").trim() {
                return Err("expectation not met of being different ".to_string());
            }
        }
        // match should be equal to
        cond if cond.starts_with("== ") => {
            // TODO handle col_type here, add int support
            if target != cond.strip_prefix("== ").unwrap_or("").trim() {
                return Err("expectation not met of being equal ".to_string());
            }
        }
        // match should be greater or smaller than x
        cond if cond.starts_with("> ") || cond.starts_with("< ") => {
            let exp: f64 = cond[2..].parse().map_err(|e| format!("{}", e))?;
            let the_num: f64 = target.parse().map_err(|e| format!("{}", e))?;

            let is_greater = condition.starts_with("> ");
            if (is_greater && the_num <= exp) || (!is_greater && the_num >= exp) {
                return Err(format!(
                    "expectation not met of numeric value: {} {}",
                    the_num, condition
                ));
            }
        }

        // match should be greater or equal or smaller or equal than x
        cond if cond.starts_with(">= ") || cond.starts_with("<= ") => {
            let exp: f64 = cond[3..].parse().map_err(|e| format!("{}", e))?;
            let the_num: f64 = target.parse().map_err(|e| format!("{}", e))?;

            let is_greater = condition.starts_with(">= ");
            if (is_greater && the_num < exp) || (!is_greater && the_num > exp) {
                return Err(format!(
                    "expectation not met of numeric value: {} {}",
                    the_num, condition
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_empty_str() {
        assert!(eval_row_condition(
            "hello".to_string(),
            "non-empty-str".to_string(),
            "varchar".to_string()
        )
        .is_ok());
        assert!(eval_row_condition(
            " ".to_string(),
            "non-empty-str".to_string(),
            "varchar".to_string()
        )
        .is_ok());
        assert!(eval_row_condition(
            "".to_string(),
            "non-empty-str".to_string(),
            "varchar".to_string()
        )
        .is_err());
    }

    #[test]
    fn test_empty_str() {
        assert!(eval_row_condition(
            "".to_string(),
            "empty-str".to_string(),
            "varchar".to_string()
        )
        .is_ok());
        assert!(eval_row_condition(
            "hello".to_string(),
            "empty-str".to_string(),
            "varchar".to_string()
        )
        .is_err());
    }

    #[test]
    fn test_not_equal_str() {
        assert!(eval_row_condition(
            "foo".to_string(),
            "!= bar".to_string(),
            "varchar".to_string()
        )
        .is_ok());
        assert!(eval_row_condition(
            "bar".to_string(),
            "!= bar".to_string(),
            "varchar".to_string()
        )
        .is_err());
        assert!(eval_row_condition(
            "bar".to_string(),
            "!=  bar ".to_string(),
            "varchar".to_string()
        )
        .is_err()); // Test extra spaces
    }

    #[test]
    fn test_equal_str() {
        assert!(eval_row_condition(
            "bar".to_string(),
            "== bar".to_string(),
            "varchar".to_string()
        )
        .is_ok());
        assert!(eval_row_condition(
            "foo".to_string(),
            "== bar".to_string(),
            "varchar".to_string()
        )
        .is_err());
        assert!(eval_row_condition(
            "bar".to_string(),
            "==  bar ".to_string(),
            "varchar".to_string()
        )
        .is_ok()); // Test extra spaces
    }

    #[test]
    fn test_greater_than_numeric() {
        assert!(
            eval_row_condition("10".to_string(), "> 5".to_string(), "number".to_string()).is_ok()
        );
        assert!(
            eval_row_condition("5".to_string(), "> 5".to_string(), "number".to_string()).is_err()
        );
        assert!(
            eval_row_condition("4".to_string(), "> 5".to_string(), "number".to_string()).is_err()
        );
    }

    #[test]
    fn test_less_than_numeric() {
        assert!(
            eval_row_condition("3".to_string(), "< 5".to_string(), "number".to_string()).is_ok()
        );
        assert!(
            eval_row_condition("5".to_string(), "< 5".to_string(), "number".to_string()).is_err()
        );
        assert!(
            eval_row_condition("6".to_string(), "< 5".to_string(), "number".to_string()).is_err()
        );
    }

    #[test]
    fn test_greater_than_or_equal_numeric() {
        assert!(
            eval_row_condition("10".to_string(), ">= 5".to_string(), "number".to_string()).is_ok()
        );
        assert!(
            eval_row_condition("5".to_string(), ">= 5".to_string(), "number".to_string()).is_ok()
        );
        assert!(
            eval_row_condition("4".to_string(), ">= 5".to_string(), "number".to_string()).is_err()
        );
    }

    #[test]
    fn test_less_than_or_equal_numeric() {
        assert!(
            eval_row_condition("3".to_string(), "<= 5".to_string(), "number".to_string()).is_ok()
        );
        assert!(
            eval_row_condition("5".to_string(), "<= 5".to_string(), "number".to_string()).is_ok()
        );
        assert!(
            eval_row_condition("6".to_string(), "<= 5".to_string(), "number".to_string()).is_err()
        );
    }

    #[test]
    fn test_invalid_number_parsing() {
        assert!(
            eval_row_condition("abc".to_string(), "> 5".to_string(), "number".to_string()).is_err()
        );
        assert!(
            eval_row_condition("5".to_string(), "> abc".to_string(), "number".to_string()).is_err()
        );
        assert!(eval_row_condition(
            "abc".to_string(),
            "<= xyz".to_string(),
            "number".to_string()
        )
        .is_err());
    }

    #[test]
    fn test_unknown_condition() {
        assert!(eval_row_condition(
            "test".to_string(),
            "some-random-condition".to_string(),
            "varchar".to_string()
        )
        .is_err());
        assert!(
            eval_row_condition("123".to_string(), "??".to_string(), "varchar".to_string()).is_err()
        );
    }
}
