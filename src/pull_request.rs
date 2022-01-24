use crate::check_status::CheckStatus;
use anyhow::{bail, Result};
use serde_json::Value;

#[derive(Debug)]
pub struct PullRequest {
    overall_status: Option<CheckStatus>,
}

impl TryFrom<&Value> for PullRequest {
    type Error = anyhow::Error;

    fn try_from(_v: &Value) -> Result<PullRequest> {
        bail!("nah")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod approved {
        use super::*;

        fn fixture() -> Value {
            serde_json::from_str(include_str!("test_fixtures/pr_approved.json")).unwrap()
        }

        #[test]
        fn loads() {
            PullRequest::try_from(&fixture()).unwrap();
        }
    }

    mod failing {
        use super::*;

        fn fixture() -> Value {
            serde_json::from_str(include_str!("test_fixtures/pr_failing.json")).unwrap()
        }

        #[test]
        fn loads() {
            PullRequest::try_from(&fixture()).unwrap();
        }
    }

    mod approved_but_failing {
        use super::*;

        fn fixture() -> Value {
            serde_json::from_str(include_str!("test_fixtures/pr_approved_but_failing.json"))
                .unwrap()
        }

        #[test]
        fn loads() {
            PullRequest::try_from(&fixture()).unwrap();
        }
    }

    mod no_checks {
        use super::*;

        fn fixture() -> Value {
            serde_json::from_str(include_str!("test_fixtures/pr_no_checks.json")).unwrap()
        }

        #[test]
        fn loads() {
            PullRequest::try_from(&fixture()).unwrap();
        }
    }
}
