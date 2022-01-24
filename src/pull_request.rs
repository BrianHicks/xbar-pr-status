use crate::check_status::CheckStatus;
use anyhow::{anyhow, Context, Result};
use serde_json::Value;

#[derive(Debug)]
pub struct PullRequest {
    overall_status: Option<CheckStatus>,
    checks: Vec<Check>,
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

    fn checks_from_commit(commit: &Value) -> Result<Vec<Check>> {
        let mut out = Vec::new();

        if let Some(contexts) = commit.pointer("/status/contexts") {
            for context in contexts
                .as_array()
                .ok_or_else(|| anyhow!("contexts was not an array"))?
            {
                out.push(
                    Check::from_context(context)
                        .context("could not load a context in the contexts array")?,
                );
            }
        }

        if let Some(suites) = commit.pointer("/checkSuites/nodes") {
            for suite in suites
                .as_array()
                .ok_or_else(|| anyhow!("suites was not an array"))?
            {
                if let Some(runs) = suite.pointer("/checkRuns/nodes") {
                    for run in runs
                        .as_array()
                        .ok_or_else(|| anyhow!("runs was not an array"))?
                    {
                        out.push(Check::from_check_run(run).context(
                            "could not a load a check run in the check suites/runs array",
                        )?)
                    }
                }
            }
        }

        Ok(out)
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
            checks: Self::checks_from_commit(commit)?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Check {
    title: String,
    status: CheckStatus,
    url: String,
}

impl Check {
    fn from_context(context: &Value) -> Result<Check> {
        Ok(Check {
            title: context
                .get("context")
                .ok_or_else(|| anyhow!("could not get context"))?
                .as_str()
                .ok_or_else(|| anyhow!("context was not a string"))?
                .into(),
            status: context
                .get("state")
                .ok_or_else(|| anyhow!("could not get state"))?
                .try_into()
                .context("could not load state from context")?,
            url: context
                .get("targetUrl")
                .ok_or_else(|| anyhow!("could not get targetUrl"))?
                .as_str()
                .ok_or_else(|| anyhow!("targetUrl was not a string"))?
                .into(),
        })
    }

    fn from_check_run(run: &Value) -> Result<Check> {
        Ok(Check {
            title: run
                .get("title")
                .ok_or_else(|| anyhow!("could not get title"))?
                .as_str()
                .ok_or_else(|| anyhow!("title was not a string"))?
                .into(),
            status: run
                .get("conclusion")
                .ok_or_else(|| anyhow!("could not get conclusion"))?
                .try_into()
                .context("could not load conclusion from context")?,
            url: run
                .get("url")
                .ok_or_else(|| anyhow!("could not get url"))?
                .as_str()
                .ok_or_else(|| anyhow!("url was not a string"))?
                .into(),
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
            assert_eq!(Some(CheckStatus::Success), fixture().overall_status)
        }

        #[test]
        fn checks() {
            assert_eq!(
                vec![
                    Check {
                        title: "Status 1".into(),
                        status: CheckStatus::Success,
                        url: "https://url".into()
                    },
                    Check {
                        title: "Status 2".into(),
                        status: CheckStatus::Success,
                        url: "https://url".into()
                    },
                    Check {
                        title: "Check 1".into(),
                        status: CheckStatus::Success,
                        url: "https://github.com/org/repo/runs/1".into()
                    },
                ],
                fixture().checks
            )
        }
    }

    mod failing {
        use super::*;

        fn fixture() -> PullRequest {
            load(include_str!("test_fixtures/pr_failing.json"))
        }

        #[test]
        fn overall_status() {
            assert_eq!(Some(CheckStatus::Failure), fixture().overall_status)
        }
    }

    mod approved_but_failing {
        use super::*;

        fn fixture() -> PullRequest {
            load(include_str!("test_fixtures/pr_approved_but_failing.json"))
        }

        #[test]
        fn overall_status() {
            assert_eq!(Some(CheckStatus::Failure), fixture().overall_status)
        }
    }

    mod no_checks {
        use super::*;

        fn fixture() -> PullRequest {
            load(include_str!("test_fixtures/pr_no_checks.json"))
        }

        #[test]
        fn overall_status() {
            assert_eq!(None, fixture().overall_status)
        }
    }
}
