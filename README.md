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

### Media-type examples

The `examples` annotation maps names to external URLs or explicit inline values:

```oal
res /items on post : <{}> `examples: {request: {value: {name: "sample"}}}`
    -> <status=200, {}> `examples: {inline: {value: {id: 1}}, remote: "examples/item.json"}`;
```

A URL string retains the existing shorthand and produces OpenAPI `externalValue`.
A mapping containing only `value` produces OpenAPI `value`, preserving objects,
arrays, strings, numbers, booleans and explicit `null`. To embed a URL as a string
value, write `{value: "https://example.test/item"}` rather than the URL shorthand.
Example names are strings; quote names containing whitespace or punctuation,
and names such as `"422"`, `"true"` or `"null"` that YAML would otherwise parse
as non-string values. Previously ignored malformed entries now produce errors.
Within backtick-delimited annotations, encode a literal backtick in a quoted
string as `\u0060`; use `\n` for an embedded newline.

Examples may annotate a schema or a content expression. Content-level examples
take precedence; when absent, the schema's examples are used. An explicit empty
mapping on the content suppresses that fallback. Malformed entries, YAML tags,
non-finite numbers and non-string object keys produce diagnostics instead of
silently losing example data. Inline values must be representable as JSON.

## Experimental: WebAssembly support
Release to WebAssembly requires the installation of [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/).

```
make wasm
```
