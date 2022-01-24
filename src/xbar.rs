use crate::check_status::CheckStatus;

#[derive(Debug, PartialEq)]
pub enum Status {
    SuccessAndApproved,
    Success,
    Pending,
    Failure,
    Unknown,
    NeedsAttention,
    Error,
    Queued,
}

impl From<&CheckStatus> for Status {
    fn from(status: &CheckStatus) -> Self {
        match &status {
            CheckStatus::Error => Status::Error,
            CheckStatus::Expected => todo!("What does expected status mean?"),
            CheckStatus::Failure => Status::Failure,
            CheckStatus::Pending => Status::Pending,
            CheckStatus::Success => Status::Success,
            CheckStatus::ActionRequired => Status::NeedsAttention,
            CheckStatus::TimedOut => Status::Error,
            CheckStatus::Cancelled => Status::Error,
            CheckStatus::Neutral => Status::Success,
            CheckStatus::Skipped => Status::Success,
            CheckStatus::StartupFailure => Status::Error,
            CheckStatus::Stale => Status::Error,
        }
    }
}
