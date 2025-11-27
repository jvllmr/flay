# flay

> [!IMPORTANT]
> This project is in an early development stage and probably should not be used in production

A utility for bundling and treeshaking your python applications and building OCI container images out of them.

One day I tasked myself with building a docker container for a python project I was working on thought it would be cool if I could further optimize the footprint of python-based containers. This cannot be too difficult I thought...

## What it does

1. Discover imports and copy the module to bundle with its dependencies into a new directory
2. Strip unused code from the bundled modules
3. Finished! Now you can use your bundled module from the new directory

## Goals

- Smaller image sizes
- Smaller layer sizes which result in faster updates
- Improved runtime performance by removing unused code and imports
- Safer containers by removing unused code which could become harmful

## Usage

### Installation

Flay needs to be run from inside the environment where the target module is located. Therefore `pipx` or the flay container image can only be used if all needed modules are located inside the current working directory.

It is recommended to install flay inside your project with your favored python package manager. The package name is `flay`.

### CLI usage

Right now there is only one command in flay:

```shell
# flay bundle <module_spec>
flay bundle flay
```

### Configuration

There are multiple ways to configure flay:

- CLI options
- `tool.flay` section in `pyproject.toml`
- env vars or `.env` file with `FLAY_` prefix
- `flay.toml` in cwd
- `flay.json` in cwd
- `flay.yaml` in cwd

Here is a quick reference of what can be configured with flay and the default values:

```yaml
# flay.yaml

# the output path to put the bundled modules in
output_path: /flayed

# if package metadata should be bundled
bundled_metadata: true

# whether the treeshake step should be run
treeshake: true

# additional non-python resources to include (e.g. html templates)
# accepts a mapping with a module as key and a glob pattern as value
resources: {}

# Import aliases mapping. Useful for patching dynamic imports. Absolute paths for symbols are required.
import-aliases: {}

# List of symbols that should be preserved at all cost. Absolute paths for symbols are required.
preserve-symbols: []

# A list of decorators without side-effects that can be safely removed. Absolute paths for symbols are required.
safe-decorators: []
```
