use anyhow::{Context, Result};
use clap::Parser;
use graphql_client::{GraphQLQuery, Response};
use reqwest::blocking::Client;
use reqwest::header;

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
    env_logger::Builder::from_env("XBAR_PR_STATUS_LOG").init();

    if let Err(err) = try_main() {
        println!("{:?}", err);
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let config = Config::parse();

    let prs =
        PullRequests::fetch(config.github_api_token).context("could not fetch pull requests")?;

    let top_line: Vec<String> = Vec::new();
    let menu_lines: Vec<String> = Vec::new();

    for pr_opt in prs.viewer.pull_requests.nodes.unwrap_or_else(|| Vec::new()) {
        let pr = pr_opt.context("got a null PR")?;
        let commits = pr.commits.nodes.context("got a null list of commits")?;
        let commit = match commits.get(0) {
            Some(Some(node)) => &node.commit,
            Some(None) => anyhow::bail!("got a null commit"),
            None => anyhow::bail!("got a null list of commits"),
        };

        println!("{:#?}", commit);
    }

    Ok(())
}

type URI = String;

type DateTime = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/github.schema.graphql",
    query_path = "src/pull_requests.graphql",
    response_derives = "Debug"
)]
struct PullRequests;

impl PullRequests {
    fn fetch(api_token: String) -> Result<pull_requests::ResponseData> {
        let query = Self::build_query(pull_requests::Variables {});

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
            .json(&query)
            .send()
            .context("could not request data from GitHub's API")?;

        let data: Response<pull_requests::ResponseData> = response
            .json()
            .context("could not deserialize pull requests from response")?;

        if let Some(errs) = data.errors {
            for err in errs {
                log::error!("{}", err)
            }
        }

        match data.data {
            Some(data) => Ok(data),
            None => anyhow::bail!("there was no data in the response"),
        }
    }
}
