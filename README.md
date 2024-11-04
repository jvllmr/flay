# flay

> [!IMPORTANT]
> This project is in a very experimental/alpha stage and can definitely not be used in a productive way so far

A utility for bundling, treeshaking and minifying your python modules and scripts and building OCI container images out of them.

One day I was tasked with building a docker container for a python project and was frustrated with the fact that I have to put all the bloat into it. This cannot be too difficult I thought...

## Goals

### Step 1

- a working pipeline for bundling, treeshaking and minifying modules
- a working CLI with no configuration effort for simple projects

### Step 2

- support for shared objects (i.e. rust based libraries such as pydantic or orjson)
- support for modules with external files such as HTML, CSS, JavaScript...
- support for bundling into "one-file"-scripts as far as possible

### Step 3

- automatically build OCI container images from your project by integrating buildah and possibly other building tools as building backends with presets provided by flay or bring your own ???
- PEP517 compliant build backend for integration with other tooling

## Comparison to other tools

### snakepack

TODO...

### pyodide-pack

TODO...

### pyinstaller

TODO...

### nukita

TODO...
