# Contribution Guidelines

First of all, thank you for taking your time to contribute to Wasmi!

Reading these contribution guidelines is your best initial step towards
successfully driving the development of Wasmi forward with your own ideas
and use cases.

## Code of Conduct

Please respect our [code of conduct](./CODE_OF_CONDUCT.md) in every
communication and discussion related to the Wasmi project.

## I don't want to contribute, I just have some questions

For technical questions about Wasmi feel free to contact
us via one of the following communication channels:

- GitHub Discussions
    - Write a [**new GitHub Discussions post**](https://github.com/wasmi-labs/wasmi/discussions/new).
    - For simple questions around usage or development of Wasmi.
- Polkadot Forums
    - Write a new post in the [**Polkadot Forum**](https://forum.polkadot.network/).
    - For public Pokadot, smart contract, ink! or `pallet-contracts`
      related Wasmi questions.
- GitHub Issue
    - Write a [**new GitHub issue**](https://github.com/wasmi-labs/wasmi/issues/new).
    - To initiate a technical (design) discussion or debate.
- Element Chat
    - Server: `matrix.parity.io`
    - Channel: `#substrate-wasm-smart-contracts:matrix.parity.io`
    - Reach out to us there especially for non-public details and business
      related (technical) support or general questions about Wasm smart contract
      execution.

## Communication & Language

In an open source project communication is one of the most important things.
People from all around the globe with different cultures and native tongues
come together to work towards a common goal.

English naturally is the language of choice for developing and communicating
technicalities concerning Wasmi. If you feel like your English skills are not
on par to properly communicate your intent don't feel ashamed to use any of
the well known translators in order to make everyone's lives simpler.
Feeding properly articulated sentences in your language to an established
translation engine usually yields good translation results.

## Feature Development

Before developing a new feature for the Wasmi interpreter on your own
we recommend checking in on the maintainers via a GitHub issue to discuss
your proposed feature in technical details.
Maintainers usually have a fundamental understanding of the codebase and
therefore might know simpler ways to achieve the same thing.

A good feature request issue consists of a problem description of the current
situation, a motivating example, the wording for the proposed feature or
change and optionally alternative designs and known issues.

## Setup

We are using a recent version of stable Rust.
Please make sure that you are having a proper stable Rust installation on your
system. We recommend using [`rustup`](https://rustup.rs/) to set you up.

For local CI runs you also are required to have the following `rustup`
components installed via `rustup component add`:

- `cargo`: The Rust build system and package manager.
- `clippy`: The Rust linter.
- `fmt`: For automatic formatting of Rust code.
- `docs`: For automatic Rust documentation generation.

Furthermore you are going to need `git` version control on your system which
you usually can install via your package manager.

Checkout the Wasmi repository using
```
git clone git@github.com:paritytech/wasmi.git
```
And develop your feature, bug fix, or miscellaneous changes inside of a feature
branch
```
git checkout -b $your-branch-name
```

## Testing

Before pushing your changes to the main repository as a PR please run the
`./scripts/run-local-ci.sh` script locally on your machine in order to make sure
that your PR is in an acceptable state.
Also this reduces resource strain on the expensive CI routines that run on every
opened PR.

If you struggle to get your PR into a shape that our CI accepts please feel
free to reach out to the maintainers and provide them with the information
they need in order to help unblock you.

### Fuzz Testing

Run Wasmi fuzz tests using the following command:

```
cargo +nightly fuzz run <target>
```
Where `<target>` is the name of any of the files under `fuzz/fuzz_targets`
directory. Unfortunately `+nightly` is required because the `cargo fuzz` tool
does not work on the stable Rust channel.

## Optimizations

If you are working on changes that are going to optimize any part of the Wasmi
interpreter please provide proper benchmarks and make sure that the optimized
code parts are properly tested.

Compare benchmarks between `master` branch and your PR as follows:
```
git checkout master
pushd crates/wasmi
cargo bench --bench benches -- --save-baseline master
git checkout $YOUR_PR
cargo bench --bench benches -- --baseline master
popd
```
This way you can ensure locally if your optimization techniques actually
improved the performance in the expected way.

**Note:** We won't merge PRs that regress performance without proper reasoning.
A proper reason is to fix a security issue that cannot be fixed otherwise.

## Commits

### Commit Signing

We require all commits to be signed. GitHub has a [pretty decent documentation]
about how to setup signing your Git commits.

[pretty decent documentation]:
https://docs.github.com/en/authentication/managing-commit-signature-verification/signing-commits

### Commit Messages

We loosely follow [semantic commit messages] without the `<type>` tags.
You are free to add those commit tags though and if you do please follow
the linked guidelines.
In any case please use decent language, short and precise commit messages,
and elaborate in case a commit needs technical explanation in isolation.

[semantic commit messages]:
https://gist.github.com/joshbuchea/6f47e86d2510bce28f8e7f42ae84c716
