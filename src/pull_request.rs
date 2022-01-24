use crate::check_status::CheckStatus;
use crate::navigate_value::NavigateValue;
use crate::xbar;
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, FixedOffset};
use serde_json::Value;

#[derive(Debug)]
pub struct PullRequest {
    title: String,
    url: String,
    updated_at: DateTime<FixedOffset>,
    approved: bool,
    queued: bool,
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

        for suite in commit.get_array("/checkSuites/nodes")? {
            for run in suite.get_array("/checkRuns/nodes")? {
                out.push(
                    Check::from_check_run(run)
                        .context("could not a load a check run in the check suites/runs array")?,
                )
            }
        }

        Ok(out)
    }

    fn approved_from_pr(pr: &Value) -> Result<bool> {
        match pr.pointer("/latestOpinionatedReviews/nodes/0/state") {
            Some(state) => Ok(state
                .as_str()
                .ok_or_else(|| anyhow!("approval state was not a string"))?
                == "APPROVED"),
            None => Ok(false),
        }
    }

    fn queued_from_pr(pr: &Value) -> Result<bool> {
        Ok(!pr
            .get("autoMergeRequest")
            .ok_or_else(|| anyhow!("autoMergeRequest was missing"))?
            .is_null())
    }

    fn status(&self) -> xbar::Status {
        match &self.overall_status {
            None => xbar::Status::Unknown,
            Some(CheckStatus::Success) => {
                if self.queued {
                    xbar::Status::Queued
                } else if self.approved {
                    xbar::Status::SuccessAndApproved
                } else {
                    xbar::Status::Success
                }
            }
            Some(other) => other.into(),
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
            title: pr.get_str("/title")?.into(),
            url: pr.get_str("/url")?.into(),
            updated_at: DateTime::parse_from_rfc3339(pr.get_str("/updatedAt")?)
                .context("updatedAt doesn't match the RFC3339 format")?,
            approved: Self::approved_from_pr(pr)?,
            queued: Self::queued_from_pr(pr)?,
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
            title: context.get_str("/context")?.into(),
            status: context
                .get_str("/state")?
                .try_into()
                .context("could not load state from context")?,
            url: context.get_str("/targetUrl")?.into(),
        })
    }

    fn from_check_run(run: &Value) -> Result<Check> {
        Ok(Check {
            title: run.get_str("/title")?.into(),
            status: run
                .get_str("/conclusion")?
                .try_into()
                .context("could not load conclusion from context")?,
            url: run.get_str("/url")?.into(),
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
        fn title() {
            assert_eq!("Title".to_string(), fixture().title)
        }

        #[test]
        fn url() {
            assert_eq!(
                "https://github.com/org/repo/pull/1".to_string(),
                fixture().url
            )
        }

        #[test]
        fn approved() {
            assert_eq!(true, fixture().approved)
        }

        #[test]
        fn queued() {
            assert_eq!(false, fixture().queued)
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

        #[test]
        fn status() {
            assert_eq!(xbar::Status::SuccessAndApproved, fixture().status())
        }
    }

    mod failing {
        use super::*;

        fn fixture() -> PullRequest {
            load(include_str!("test_fixtures/pr_failing.json"))
        }

        #[test]
        fn title() {
            assert_eq!("Title".to_string(), fixture().title)
        }

        #[test]
        fn url() {
            assert_eq!(
                "https://github.com/org/repo/pull/1".to_string(),
                fixture().url
            )
        }

        #[test]
        fn approved() {
            assert_eq!(false, fixture().approved)
        }

        #[test]
        fn queued() {
            assert_eq!(false, fixture().queued)
        }

        #[test]
        fn overall_status() {
            assert_eq!(Some(CheckStatus::Failure), fixture().overall_status)
        }

        #[test]
        fn checks() {
            assert_eq!(
                vec![
                    Check {
                        title: "Check 1".into(),
                        status: CheckStatus::Failure,
                        url: "https://github.com/org/repo/runs/1".into()
                    },
                    Check {
                        title: "Check 2".into(),
                        status: CheckStatus::Cancelled,
                        url: "https://github.com/org/repo/runs/2".into()
                    },
                ],
                fixture().checks
            )
        }

        #[test]
        fn status() {
            assert_eq!(xbar::Status::Failure, fixture().status())
        }
    }

    mod approved_but_failing {
        use super::*;

        fn fixture() -> PullRequest {
            load(include_str!("test_fixtures/pr_approved_but_failing.json"))
        }

        #[test]
        fn title() {
            assert_eq!("Title".to_string(), fixture().title)
        }

        #[test]
        fn url() {
            assert_eq!(
                "https://github.com/org/repo/pull/1".to_string(),
                fixture().url
            )
        }

        #[test]
        fn approved() {
            assert_eq!(true, fixture().approved)
        }

        #[test]
        fn queued() {
            assert_eq!(false, fixture().queued)
        }

        #[test]
        fn overall_status() {
            assert_eq!(Some(CheckStatus::Failure), fixture().overall_status)
        }

        #[test]
        fn checks() {
            assert_eq!(
                vec![
                    Check {
                        title: "Check 1".into(),
                        status: CheckStatus::Failure,
                        url: "https://github.com/org/repo/runs/1".into()
                    },
                    Check {
                        title: "Check 2".into(),
                        status: CheckStatus::Failure,
                        url: "https://github.com/org/repo/runs/2".into()
                    },
                ],
                fixture().checks
            )
        }

        #[test]
        fn status() {
            assert_eq!(xbar::Status::Failure, fixture().status())
        }
    }

    mod no_checks {
        use super::*;

        fn fixture() -> PullRequest {
            load(include_str!("test_fixtures/pr_no_checks.json"))
        }

        #[test]
        fn title() {
            assert_eq!("Title".to_string(), fixture().title)
        }

        #[test]
        fn url() {
            assert_eq!(
                "https://github.com/org/repo/pull/1".to_string(),
                fixture().url
            )
        }

        #[test]
        fn approved() {
            assert_eq!(false, fixture().approved)
        }

        #[test]
        fn queued() {
            assert_eq!(false, fixture().queued)
        }

        #[test]
        fn overall_status() {
            assert_eq!(None, fixture().overall_status)
        }

        #[test]
        fn checks() {
            let empty: Vec<Check> = Vec::new();
            assert_eq!(empty, fixture().checks)
        }

        #[test]
        fn status() {
            assert_eq!(xbar::Status::Unknown, fixture().status())
        }
    }
}
