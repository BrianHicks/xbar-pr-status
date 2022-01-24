mod check_status;
mod navigate_value;
mod pull_request;
mod xbar;

use crate::navigate_value::NavigateValue;
use crate::pull_request::PullRequest;
use anyhow::{bail, Context, Result};
use clap::Parser;
use reqwest::blocking::Client;
use reqwest::header;
use serde_json::{json, Value};

#[derive(Debug, Parser)]
#[clap(about, author)]
pub struct Config {
    // TODO: document scopes
    #[clap(env = "GITHUB_API_TOKEN")]
    github_api_token: String,

    /// Emoji to use when CI is passing and the PR is approved
    #[clap(long, env = "SUCCESS_AND_APPROVED_EMOJI", default_value = "🌝")]
    success_and_approved_emoji: String,

    /// Emoji to use when CI is passing but the PR is not yet approved
    #[clap(long, env = "SUCCESS_EMOJI", default_value = "🌕")]
    success_emoji: String,

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
}

fn main() {
    env_logger::Builder::from_env("XBAR_PR_STATUS_LOG").init();

    if let Err(err) = try_main() {
        println!("{:?}", err);
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let config = Config::parse();

    let prs = fetch(&config.github_api_token).context("could not fetch pull requests")?;

    let mut top_line: Vec<&str> = Vec::new();
    let mut menu_lines: Vec<String> = Vec::new();

    for pr_value in prs.get_array("/data/viewer/pullRequests/nodes")? {
        let pr = match PullRequest::try_from(pr_value).context("could not load a Pull Request") {
            Ok(pr) => pr,
            Err(err) => {
                log::debug!("{:#?}", pr_value);
                return Err(err).context("could not load a Pull Request");
            }
        };
        top_line.push(config.emoji_for(pr.status()));
        menu_lines.push(pr.to_xbar_menu(&config));
    }

    print!("{}\n---\n{}\n", top_line.join(""), menu_lines.join("\n"));

    Ok(())
}

fn fetch(api_token: &str) -> Result<Value> {
    let client = Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .context("could not build the HTTP client")?;

    let response = client
        .post("https://api.github.com/graphql")
        .header(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", api_token))
                .context("could not create an Authorization header from the specified token")?,
        )
        .json(&json!({ "query": include_str!("pull_requests.graphql") }))
        .send()
        .context("could not request data from GitHub's API")?;

    let body: Value = response.json().context("could not read JSON body")?;

    if let Some(value) = body.pointer("errors") {
        match value {
            Value::Null => (),
            Value::Array(errs) => {
                for err in errs {
                    log::error!("{}", err);
                }
            }
            _ => bail!("errors was not an array"),
        }
    }

    Ok(body)
}

impl Config {
    pub fn emoji_for(&self, status: xbar::Status) -> &str {
        match status {
            xbar::Status::SuccessAndApproved => &self.success_and_approved_emoji,
            xbar::Status::Success => &self.success_emoji,
            xbar::Status::Pending => &self.pending_emoji,
            xbar::Status::Failure => &self.failure_emoji,
            xbar::Status::Unknown => &self.unknown_emoji,
            xbar::Status::NeedsAttention => &self.needs_attention_emoji,
            xbar::Status::Error => &self.error_emoji,
            xbar::Status::Queued => &self.queued_emoji,
        }
    }
}
