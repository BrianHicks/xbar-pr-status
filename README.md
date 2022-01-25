# xbar PR status

This is an [xbar](https://xbarapp.com/) plugin that creates a view of your open GitHub PRs and their CI/approval/automerge status.
The menubar item opens to a list of linked PRs, with further expansion to a list of checks on each PR.

## Installing

This repo is managed using Nix, so you can do something like:

```sh
cachix use xbar-pr-status # if you have cachix installed
nix-env -if https://github.com/BrianHicks/xbar-pr-status/archive/refs/heads/main.tar.gz
```

If you have a Rust toolchain set up, you can also clone this repo and run `cargo build`.

### Setting Up

[Create a personal GitHub access token in your developer settings](https://github.com/settings/tokens).
It needs the `repo` (all) and `read:user` scopes (you can see the exact query we make at `src/pull_requests.graphql`.)

After you install xbar-pr-status, create a new plugin like `~/Library/Application Support/xbar/plugins/prs.5m.sh` that looks like this:

```bash
#!/usr/bin/env bash
xbar-pr-status ghp_AAAAAAAAAAA
```

Make that file executable (`chmod +x`) and tell xbar to reload all plugins to pick it up.
(You should also be able to run it yourself to check if things look OK.)

If you want to customize the emojis used, run `xbar-pr-status --help` to see the options available to you.

## License

xbar-pr-status is licensed under [The Hippocratic License](https://firstdonoharm.dev/), version 3.0. See LICENSE in the source for the applicable modules to this project.
