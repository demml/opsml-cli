# Opsml-cli

[![Lints-Tests](https://github.com/shipt/opsml-cli/actions/workflows/lint-testing.yml/badge.svg)](https://github.com/shipt/opsml-cli/actions/workflows/lint-testing.yml)

This package provides a python-installable CLI written in `Rust` for interacting with [opsml](https://github.com/shipt/opsml).

## Installation

### Using Poetry

```bash
poetry add opsml-cli
```

### Using Pip

```bash
pip install opsml-cli
```

## Commands

To get a list of commands, run `opsml-cli help`.

### Listing Cards

```console

$ opsml-cli list-cards --registry model
```

### Downloading Model

```console
# Download model only

$ opsml-cli download-model --name {{model}} --repository {{repository}} --version {{version}}  # name repository version
$ opsml-cli download-model --uid {{uid}} # model from uid

# Download onnx version of model

$ opsml-cli download-model --name model --repository {{repository}} --version 1.0.0  --onnx

# Download model and preprocessor (if available)
opsml-cli download-model --name model -- version 1.0.0 --repository {{repository}} --onnx --preprocessor
```

## Contributing
If you'd like to contribute, be sure to check out our [contributing guide](./CONTRIBUTING.md)!

Thanks goes to these phenomenal [projects and people](./ATTRIBUTIONS.md) and people for creating a great foundation to build from!

<a href="https://github.com/demml/opsml-cli/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=demml/opsml-cli" />
</a>

