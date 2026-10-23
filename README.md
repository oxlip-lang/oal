![Build](https://img.shields.io/github/actions/workflow/status/oxlip-lang/oal/ci.yml?branch=master)
[![License](https://img.shields.io/badge/license-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

# The Oxlip API Language
Oxlip is a high-level functional programming language for designing [OpenAPI](https://www.openapis.org/what-is-openapi) definitions.
As an [Interface Description Language](https://en.wikipedia.org/wiki/Interface_description_language), it is not general purpose.
The motivation is to alleviate the pain of managing OpenAPI in JSON or YAML by hand and at scale.
Oxlip defines algebraic abstractions over [REST](https://en.wikipedia.org/wiki/Representational_state_transfer) concepts, not too dissimilar to [Sass/SCSS over CSS](https://sass-lang.com/).

There are pros and cons to both _API-design-first_ and OpenAPI generated from implementation.
As OpenAPI is better produced or consumed by machines rather than humans, Oxlip tries to help _API-design-first_ teams with better tooling.

## [Documentation](https://www.oxlip-lang.org/)

## [Playground](https://oxlip-lang.github.io/oxlip-playground)

## Installation
This step requires a [local Rust and Cargo installation](https://doc.rust-lang.org/cargo/getting-started/installation.html).

```
make install
```
Optional: a [VSCode language extension](https://github.com/oxlip-lang/oal-vscode) is available for syntax highlighting and IDE capabilities.

## Usage
```
    oal-cli [OPTIONS]

OPTIONS:
    -b, --base <BASE>        The relative URL to a base OpenAPI description
    -c, --conf <CONFIG>      The path to the configuration file
    -h, --help               Print help information
    -m, --main <MAIN>        The relative URL to the main program
    -t, --target <TARGET>    The relative URL to the target OpenAPI description
```

### Compiling the example program
```
oal-cli --conf examples/oal.toml
```

## Experimental: WebAssembly support
Release to WebAssembly requires the installation of [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/).

```
make wasm
```

## Operation Security

Declare OpenAPI security requirements on a transfer:

```oal
# security: [{api_key: []}, {oauth: [read, write]}]
let protected = get -> {};
res /items on protected;

# security: []
let public = get -> {};
res /health on public;
```

Requirements in the array are alternatives (OR). Schemes within one requirement
are all required (AND), for example `security: [{api_key: [], oauth: [read]}]`.
An empty requirement permits anonymous access: `security: [{}, {oauth: [read]}]`.
Omitting the annotation inherits document-level security; `security: []`
explicitly disables it for the operation. A later security annotation replaces
earlier requirements rather than concatenating alternatives, including when
the later annotation is empty.

Scheme names and scopes must be strings, with a scope array for every scheme.
Malformed values produce diagnostics; YAML-only tags are rejected. Use empty
scope arrays for schemes without OAuth/OpenID scopes. Scheme definitions remain
in `components.securitySchemes` of the CLI's base OpenAPI document; this feature
does not add scheme declarations to OAL or a base-document argument to WASM.
References and scopes are emitted as supplied, not checked against that base.
These annotations describe authentication requirements; they do not enforce it.
