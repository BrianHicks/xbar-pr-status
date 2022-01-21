use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(about, author)]
struct Config {
    // TODO: document scopes
    #[clap(env = "GITHUB_API_TOKEN")]
    github_api_token: String,

    /// Emoji to use when CI is passing and the PR is approved
    #[clap(long, env = "SUCCESS_AND_APPROVED_EMOJI", default_value = "🌝")]
    success_and_approved_emoji: String,

    /// Emoji to use when CI is passing but the PR is not yet approved
    #[clap(long, env = "SUCCESS_AWAITING_APPROVAL_EMOJI", default_value = "🌕")]
    success_awaiting_approval_emoji: String,

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

    /// Emoji to use when the PR enters the merge queue
    #[clap(long, env = "QUEUED_EMOJI", default_value = "✨")]
    queued_emoji: String,
}

fn main() {
    if let Err(err) = try_main() {
        println!("{:#?}", err);
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let config = Config::parse();

    println!("{:#?}", config);

    Ok(())
}
