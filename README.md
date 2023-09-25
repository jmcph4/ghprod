# ghprod #

[![standard-readme compliant](https://img.shields.io/badge/standard--readme-OK-green.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)

`ghprod`

`ghprod` (short for **G**it**H**ub **prod**uctivity tool) is a tool for quantifying and querying the productivity of software development hosted on GitHub. It allows reasoning about the performance of both individual developers but also teams of developers.

Some of `ghprod`'s current functionality includes:

 - Calculate mean PR duration time (in days) for a given developer
 - Calculate median PR duration time (in days) for a given developer
 - Both unauthenticated and authenticated API access (via the `--api-secret` flag)
 - Automatic depagination (with sleeps built-in)

## Table of Contents

- [Security](#security)
- [Background](#background)
- [Install](#install)
- [Usage](#usage)
- [API](#api)
- [Maintainers](#maintainers)
- [Contributing](#contributing)
- [License](#license)

## Security

Presently, `ghprod` has a very low sensitivity to security issues due to its relatively simple and benign nature; however, if this changes, a formal security policy will be instated (via a `SECURITY.md` file).

## Background

## Install

```
$ git clone git@github.com:jmcph4/ghprod.git
$ cd ghprod
$ cargo build --release
```

## Usage

Usage information is available via the usual route:

```
$ ghprod --help
Usage: ghprod [OPTIONS] <OWNER> <REPO> <COMMAND>

Commands:
  solo
  help  Print this message or the help of the given subcommand(s)

Arguments:
  <OWNER>
  <REPO>

Options:
  -a, --api-secret <API_SECRET>
  -p, --pull-request-terminating-state <PULL_REQUEST_TERMINATING_STATE>
  -h, --help                                                             Print help
  -V, --version                                                          Print version
```

```
$ ghprod jmcph4 ghprod solo jmcph4 median_pr_duration
```

For example, @gakonst's mean PR duration for the paradigmxyz/reth repository (denominated in days):

```
$ ghprod paradigmxyz reth solo gakonst mean_pr_duration
18.33052837401796
```

If you hit the public rate limits, specify a [personal access token](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens) via the `--api-secret` flag:

```
$ ghprod paradigmxyz reth --api-secret <MY_GITHUB_TOKEN> solo gakonst mean_pr_duration
18.33052837401796
```

## API

## Maintainers

[@jmcph4](https://github.com/jmcph4)

## Contributing

See [the contributing file](CONTRIBUTING.md)!

PRs accepted.

Small note: If editing the README, please conform to the [standard-readme](https://github.com/RichardLitt/standard-readme) specification.

## License

MIT © 2023 Jack McPherson.

