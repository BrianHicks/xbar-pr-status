use anyhow::{anyhow, bail, Result};
use serde_json::Value;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub enum CheckStatus {
    Error,
    Expected,
    Failure,
    Pending,
    Success,

    // additional statuses for check conclusions
    ActionRequired,
    TimedOut,
    Cancelled,
    Neutral,
    Skipped,
    StartupFailure,
    Stale,
}

impl FromStr for CheckStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "ACTION_REQUIRED" => Ok(Self::ActionRequired),
            "CANCELLED" => Ok(Self::Cancelled),
            "ERROR" => Ok(Self::Error),
            "EXPECTED" => Ok(Self::Expected),
            "FAILURE" => Ok(Self::Failure),
            "NEUTRAL" => Ok(Self::Neutral),
            "PENDING" => Ok(Self::Pending),
            "SKIPPED" => Ok(Self::Skipped),
            "STALE" => Ok(Self::Stale),
            "STARTUP_FAILURE" => Ok(Self::StartupFailure),
            "SUCCESS" => Ok(Self::Success),
            "TIMED_OUT" => Ok(Self::TimedOut),
            _ => bail!("got unexpected value {} as a CheckStatus", s),
        }
    }
}

impl TryFrom<&str> for CheckStatus {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self> {
        Self::from_str(s)
    }
}

impl TryFrom<&Value> for CheckStatus {
    type Error = anyhow::Error;

    fn try_from(v: &Value) -> Result<Self> {
        Self::from_str(
            v.as_str()
                .ok_or_else(|| anyhow!("value passed to CheckStatus was not a string"))?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod from_str {
        use super::*;

        #[test]
        fn error() {
            assert_eq!(CheckStatus::Error, CheckStatus::from_str("ERROR").unwrap())
        }

        #[test]
        fn expected() {
            assert_eq!(
                CheckStatus::Expected,
                CheckStatus::from_str("EXPECTED").unwrap()
            )
        }

        #[test]
        fn failure() {
            assert_eq!(
                CheckStatus::Failure,
                CheckStatus::from_str("FAILURE").unwrap()
            )
        }

        #[test]
        fn pending() {
            assert_eq!(
                CheckStatus::Pending,
                CheckStatus::from_str("PENDING").unwrap()
            )
        }

        #[test]
        fn success() {
            assert_eq!(
                CheckStatus::Success,
                CheckStatus::from_str("SUCCESS").unwrap()
            )
        }

        #[test]
        fn action_required() {
            assert_eq!(
                CheckStatus::ActionRequired,
                CheckStatus::from_str("ACTION_REQUIRED").unwrap()
            )
        }

        #[test]
        fn timed_out() {
            assert_eq!(
                CheckStatus::TimedOut,
                CheckStatus::from_str("TIMED_OUT").unwrap()
            )
        }

        #[test]
        fn cancelled() {
            assert_eq!(
                CheckStatus::Cancelled,
                CheckStatus::from_str("CANCELLED").unwrap()
            )
        }

        #[test]
        fn neutral() {
            assert_eq!(
                CheckStatus::Neutral,
                CheckStatus::from_str("NEUTRAL").unwrap()
            )
        }

        #[test]
        fn skipped() {
            assert_eq!(
                CheckStatus::Skipped,
                CheckStatus::from_str("SKIPPED").unwrap()
            )
        }

        #[test]
        fn startup_failure() {
            assert_eq!(
                CheckStatus::StartupFailure,
                CheckStatus::from_str("STARTUP_FAILURE").unwrap()
            )
        }

        #[test]
        fn stale() {
            assert_eq!(CheckStatus::Stale, CheckStatus::from_str("STALE").unwrap())
        }

        #[test]
        fn other_string() {
            assert_eq!(
                "got unexpected value NOPE as a CheckStatus",
                CheckStatus::from_str("NOPE").unwrap_err().to_string()
            )
        }
    }

    mod from_value {
        use super::*;
        use serde_json::json;

        #[test]
        fn acceptable_string() {
            assert_eq!(
                CheckStatus::Success,
                CheckStatus::try_from(&json!("SUCCESS")).unwrap(),
            )
        }

        #[test]
        fn unacceptable_string() {
            assert_eq!(
                "got unexpected value NOPE as a CheckStatus",
                CheckStatus::try_from(&json!("NOPE"))
                    .unwrap_err()
                    .to_string(),
            )
        }

        #[test]
        fn bad_type() {
            assert_eq!(
                "value passed to CheckStatus was not a string",
                CheckStatus::try_from(&json!(null)).unwrap_err().to_string(),
            )
        }
    }
}
