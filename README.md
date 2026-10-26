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

## Mixed URI Segments

URI segments can combine literal text and typed parameters:

```oal
res /api/db/{'dataset str}.{'table str} on get -> {};
res /v{'version int}/files/{'name str}.json on get -> {};

let id = 'id int;
res /items/item-{id} on get -> {};
```

These produce `/api/db/{dataset}.{table}`, `/v{version}/files/{name}.json`
and `/items/item-{id}` without post-processing. Every parameter still has its
own required OpenAPI path-parameter declaration and primitive type. Literal
prefixes/suffixes, multiple parameters, URI `concat`, query parameters, URI
schema examples and generated operation IDs use the same segment structure.

Fragments within a segment must be adjacent; whitespace and comments are still
allowed inside parameter braces. Existing slash-separated syntax, including
whitespace between `/` and its parameter braces, is retained. Literal fragments
use ASCII letters/digits, `_`, `-`, `.`, and percent-encoded data, which is
preserved verbatim. Encode reserved literal braces as `%7B`/`%7D` and Unicode
as UTF-8 percent escapes; the compiler does not decode or validate those escapes.
After a parameter, `~` and `->` retain their OAL operator meaning, and the token
`on` still begins a relation even without whitespace. Use `%7E`, `%2D%3E` or
`%6Fn` if those sequences are intended as literal suffixes. A literal `/` starts
another segment; `%2F` remains part of the current segment.

## Experimental: WebAssembly support
Release to WebAssembly requires the installation of [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/).

```
make wasm
```
