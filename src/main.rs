mod check_status;
mod navigate_value;
mod pull_request;
mod xbar;

use crate::navigate_value::NavigateValue;
use crate::pull_request::PullRequest;
use anyhow::{bail, Context, Result};
use chrono::{Duration, Local};
use clap::Parser;
use reqwest::blocking::Client;
use reqwest::header;
use serde_json::{json, Value};

#[derive(Debug, Parser)]
#[clap(about, author)]
pub struct Config {
    /// A GitHub access token, created with the `repo` and `read:user` scopes.
    ///
    /// You can make one of these st https://github.com/settings/tokens
    #[clap(env = "GITHUB_API_TOKEN")]
    github_api_token: String,

    /// Ignore PRs updated last before this many days ago
    #[clap(long, env = "SINCE")]
    since: Option<i64>,

    #[clap(flatten)]
    emoji: xbar::Emoji,
}

fn main() {
    env_logger::Builder::from_env("XBAR_PR_STATUS_LOG").init();

    if let Err(err) = try_main() {
        println!("{err:?}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let config = Config::parse();
    let cutoff_opt = config.since.map(|days| Local::now() - Duration::days(days));

    let prs = fetch(&config.github_api_token).context("could not fetch pull requests")?;

    let mut top_line: Vec<&str> = Vec::new();
    let mut menu_lines: Vec<String> = Vec::new();

    if let Ok(errors) = prs.get_array("/errors") {
        for error in errors {
            log::error!("{error:#?}");
        }
    }

    for pr_value in prs.get_array("/data/viewer/pullRequests/nodes")? {
        let pr = match PullRequest::try_from(pr_value).context("could not load a Pull Request") {
            Ok(pr) => pr,
            Err(err) => {
                log::debug!("{:#?}", pr_value);
                return Err(err).context("could not load a Pull Request");
            }
        };

        if matches!(cutoff_opt, Some(cutoff) if pr.updated_at < cutoff) {
            continue;
        }

        top_line.push(config.emoji.for_status(pr.status()));
        menu_lines.push(pr.to_xbar_menu(&config.emoji));
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
            header::HeaderValue::from_str(&format!("Bearer {api_token}"))
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
