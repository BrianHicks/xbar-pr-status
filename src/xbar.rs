use crate::check_status::CheckStatus;
use anyhow::{Context, Result};
use clap::Parser;

#[derive(Debug, PartialEq)]
pub enum Status {
    SuccessAndApproved,
    SuccessAwaitingApproval(String),
    Draft,
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

#[derive(Debug, Parser)]
pub struct Emoji {
    /// Emoji to use when CI is passing and the PR is approved
    #[clap(long, env = "SUCCESS_AND_APPROVED_EMOJI", default_value = "🌝")]
    success_and_approved_emoji: String,

    /// Emoji to use when CI is passing but the PR is not yet approved
    #[clap(long, env = "SUCCESS_EMOJI", default_value = "🌕")]
    success_emoji: String,

    #[clap(long, env = "DRAFT_EMOJI", default_value = "🚧")]
    draft_emoji: String,

    /// Emoji to use when we're waiting to hear back from CI
    #[clap(long, env = "PENDING_EMOJI", default_value = "🌓")]
    pending_emoji: String,

    /// Emoji to use when CI fails
    #[clap(long, env = "FAILURE_EMOJI", default_value = "🌑")]
    failure_emoji: String,

    /// Emoji to use when there are no configured CI checks
    #[clap(long, env = "UNKNOWN_EMOJI", default_value = "🌔")]
    unknown_emoji: String,

    /// Emoji to use when CI reports an error
    #[clap(long, env = "ERROR_EMOJI", default_value = "💥")]
    error_emoji: String,

    /// Emoji to use when CI needs attention
    #[clap(long, env = "NEEDS_ATTENTION_EMOJI", default_value = "❗️")]
    needs_attention_emoji: String,

    /// Emoji to use when the PR enters the merge queue
    #[clap(long, env = "QUEUED_EMOJI", default_value = "✨")]
    queued_emoji: String,

    /// Emoji for a specific reviewer while waiting for review. Format is
    /// reviewerGithubLogin=EMOJI
    #[clap(long("reviewer-emoji"), value_parser = parse_reviewer, action = clap::ArgAction::Append)]
    reviewer_emojis: Vec<(String, String)>,

    #[clap(long, env = "DEFAULT_REVIEWER_EMOJI", default_value = "🌜")]
    default_reviewer_emoji: String,
}

impl Emoji {
    pub fn for_status(&self, status: Status) -> &str {
        match status {
            Status::SuccessAndApproved => &self.success_and_approved_emoji,
            Status::SuccessAwaitingApproval(reviewer) => self
                .reviewer_emojis
                .iter()
                .filter_map(|(name, emoji)| if name == &reviewer { Some(emoji) } else { None })
                .next()
                .unwrap_or(&self.default_reviewer_emoji),
            Status::Success => &self.success_emoji,
            Status::Draft => &self.draft_emoji,
            Status::Pending => &self.pending_emoji,
            Status::Failure => &self.failure_emoji,
            Status::Unknown => &self.unknown_emoji,
            Status::NeedsAttention => &self.needs_attention_emoji,
            Status::Error => &self.error_emoji,
            Status::Queued => &self.queued_emoji,
        }
    }
}

fn parse_reviewer(s: &str) -> Result<(String, String)> {
    let mut items = s.split('=');
    Ok((
        items
            .next()
            .with_context(|| format!("I couldn't find a reviewer name in `{}`", s))?
            .to_string(),
        items
            .next()
            .with_context(|| format!("I couldn't find a reviewer emoji in `{}`", s))?
            .to_string(),
    ))
}
