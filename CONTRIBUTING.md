# Contributing to demml/opsml-cli

## Welcome
Hello! We're glad and grateful that you're interested in contributing to `Opsml-cli` :tada:! Below you will find the general guidelines for setting up your environment and creating/submitting `pull requests`.


## Table of contents

- [Contributing to demml/opsml-cli](#contributing-to-demmlopsml-cli)
  - [Welcome](#welcome)
  - [Table of contents](#table-of-contents)
  - [Environment Setup](#environment-setup)
    - [Note](#note)
  - [Contributing Changes](#contributing-changes)
  - [Contributing TLDR](#contributing-tldr)
  - [Community Guidelines](#community-guidelines)
  - [Submitting issues/bugs](#submitting-issuesbugs)
  - [Suggesting enhancements](#suggesting-enhancements)
  - [_Thank you!_](#thank-you)


## Environment Setup
Steps:
1. Install rust (https://doc.rust-lang.org/cargo/getting-started/installation.html)
2. Create new virtual environment (pyenv is recommended) and install python (opsml-cli officially supports python 3.9 -> 3.11)
3. Activate your new virtual environment and install maturin
4. Fork `Opsml-cli` and clone your fork locally
5. Run tests locally (`cargo test`)

### Note
Because the CLI is a rust-only implementation and we generate rust binaries, you don't necessarily need to have python installed to run the CLI. However, if you want to run the CLI from source, you will need to have python installed (e.g. if you want to run `maturin develop` and `pip install -e .` locally to test)

## Contributing Changes
1. Create a new branch for your addition
   * General naming conventions (we're not picky):
      * `/username/<featureName>`: for features
      * `/username/<fixName>`: for general refactoring or bug fixes
2. Test your changes:
   * You can run formatting, lints and tests locally via `make format`, `make lints` and `make unit.tests`, respectively.
3. Submit a Draft Pull Request. Do it early and mark it `WIP` so a maintainer knows it's not ready for review just yet. You can also add a label to it if you feel like it :smile:.
4. Move the `pull_request` out of draft state.
   * Make sure you fill out the `pull_request` template (included with every `pull_request`)
5. Request review from one of our maintainers (this should happen automatically via `.github/CODEOWNERS`). 
6. Get Approval. We'll let you know if there are any changes that are needed. 
7. Merge your changes into `Opsml-cli`!

## Contributing TLDR
1. Create branch
2. Add changes
3. Test locally
4. Get your awesome work reviewed and approved by a maintainer
5. Merge
6. Celebrate!

## Community Guidelines
  1. Be Kind
    - Working with us should be a fun learning opportunity, and we want it to be a good experience for everyone. Please treat each other with respect.  
    - If something looks outdated or incorrect, please let us know! We want to make `Opsml-cli` as useful as possible. 
  2. Own Your Work
     * Creating a PR for `Opsml-cli` is your first step to becoming a contributor, so make sure that you own your changes. 
     * Our maintainers will do their best to respond to you in a timely manner, but we ask the same from you as the contributor. 

## Submitting issues/bugs

We use [GitHub issues](https://github.com/demml/opsml-cli/issues) to track bugs and suggested enhancements. You can report a bug by opening a new issue [new issue](https://github.com/demml/opsml-cli/issues/new/choose) Before reporting a bug/issue, please check that it has not already been reported, and that it is not already fixed in the latest version. If you find a closed issue related to your current issue, please open a new issue and include a link to the original issue in the body of your new one. Please include as much information about your bug as possible.

## Suggesting enhancements

You can suggest an enhancement by opening a [new feature request](https://github.com/demml/opsml-cli/issues/new?labels=enhancement&template=feature_request.yml).
Before creating an enhancement suggestion, please check that a similar issue does not already exist.

Please describe the behavior you want and why, and provide examples of how `Opsml-cli` would be used if your feature were added.

## _Thank you!_