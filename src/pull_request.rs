use crate::check_status::CheckStatus;
use anyhow::{anyhow, Context, Result};
use serde_json::Value;

#[derive(Debug)]
pub struct PullRequest {
    overall_status: Option<CheckStatus>,
}

impl PullRequest {
    fn overall_status_from_commit(commit: &Value) -> Result<Option<CheckStatus>> {
        match commit.pointer("/statusCheckRollup/state") {
            Some(state) => Ok(Some(
                state
                    .as_str()
                    .ok_or_else(|| anyhow!("state was not a string"))?
                    .try_into()
                    .context("could not load status from overall state")?,
            )),
            None => Ok(None),
        }
    }
}

impl TryFrom<&Value> for PullRequest {
    type Error = anyhow::Error;

    fn try_from(pr: &Value) -> Result<PullRequest> {
        let commit = pr
            .pointer("/commits/nodes/0/commit")
            .ok_or_else(|| anyhow!("could not get the last commit"))?;

        Ok(PullRequest {
            overall_status: Self::overall_status_from_commit(commit)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load(s: &str) -> PullRequest {
        PullRequest::try_from(&serde_json::from_str(s).unwrap()).unwrap()
    }

    mod approved {
        use super::*;

        fn fixture() -> PullRequest {
            load(include_str!("test_fixtures/pr_approved.json"))
        }

        #[test]
        fn overall_status() {
            assert_eq!(Some(CheckStatus::Success), fixture().overall_status,)
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
