mod check_status;
use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use reqwest::blocking::Client;
use reqwest::header;
use serde_json::{json, Value};
use std::str::FromStr;

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

    // anyhow::bail!("x");

    let prs = fetch(config.github_api_token).context("could not fetch pull requests")?;

    let top_line: Vec<String> = Vec::new();
    let menu_lines: Vec<String> = Vec::new();

    for pr in prs
        .pointer("/viewer/pullRequests/nodes")
        .ok_or_else(|| anyhow!("could not get PRs"))?
        .as_array()
        .ok_or_else(|| anyhow!("/viewer/pullRequests/nodes was not an array"))?
    {
        // Determine the top-level status
        let commit = pr
            .pointer("/commits/nodes/0/commit")
            .ok_or_else(|| anyhow!("could not get the last commit"))?;

        let contexts: Vec<(&str, Status)> = Vec::new();
        for context in commit
            .pointer("/status/contexts")
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| &Vec::new())
        {
            println!("{:#?}", context);
        }
        // let contexts: Result<Vec<(&str, &str)>> = commit
        //     .pointer("/status/contexts")
        //     .and_then(|v| v.as_array())
        //     .map(|a| {
        //         a.iter()
        //             .flat_map(|context| {
        //                 Ok((
        //                     context
        //                         .get("context")
        //                         .ok_or_else(|| anyhow!("context was null"))?
        //                         .as_str()
        //                         .ok_or_else(|| anyhow!("context was not a string"))?,
        //                     context
        //                         .get("state")
        //                         .ok_or_else(|| anyhow!("state was null"))?
        //                         .try_into()?,
        //                 ))
        //             })
        //             .collect::<Vec<(&str, Status)>>()
        //     });

        let overall: Option<&str> = commit
            .pointer("/statusCheckRollup/state")
            .and_then(|state| state.as_str());

        println!("{:#?}", contexts);
        println!("{:#?}", overall);
    }

    // for pr_opt in prs.viewer.pull_requests.nodes.unwrap_or_else(|| Vec::new()) {
    //     let pr = pr_opt.context("got a null PR")?;

    // Determine the top-level status
    // let commits = pr.commits.nodes.context("got a null list of commits")?;
    // let commit = match commits.get(0) {
    //     Some(Some(node)) => &node.commit,
    //     Some(None) => anyhow::bail!("got a null commit"),
    //     None => anyhow::bail!("got a null list of commits"),
    // };

    // let rollup = commit
    //     .status_check_rollup
    //     .map(|rollup| rollup.state)
    //     .unwrap_or()
    //     // .as_ref()
    //     // .map(|rollup| match rollup.state {
    //     //     pull_requests::StatusState::EXPECTED => 1,
    //     //     pull_requests::StatusState::ERROR => 0,
    //     // });
    //     ;

    // println!("{:#?}", is_approved(&pr));
    // }

    Ok(())
}

fn fetch(api_token: String) -> Result<Value> {
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

    match body.get("data") {
        // probably a better way around this than cloning but I don't know it ATM!
        Some(data) => Ok(data.clone()),
        None => bail!("there was no data in the response"),
    }
}

// fn is_approved(pr: &pull_requests::PullRequestsViewerPullRequestsNodes) -> Result<bool> {
//     let latest = pr
//         .latest_opinionated_reviews
//         .context("latest opinionated reviews was null")?;

//     let reviews = latest
//         .nodes
//         .context("latest_opinionated_reviews.nodes was null")?;

//     match reviews.get(0) {
//         Some(Some(review)) => Ok(review.state == pull_requests::PullRequestReviewState::APPROVED),
//         Some(None) => anyhow::bail!("the first review was null"),
//         None => Ok(false), // no reviews (yet!)
//     }
// }
