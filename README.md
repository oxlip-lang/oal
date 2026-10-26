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

### Base document input

The historic JavaScript `compile(oalText)` function is unchanged. A separate
WASM entry point accepts two JSON-compatible objects:

```js
const result = compileDocument(
  { source: "res /items on get -> {};" },
  {
    openapi: "3.0.3",
    info: { title: "Items API", version: "1.0.0" },
    paths: {},
    components: {
      securitySchemes: {
        bearer: { type: "http", scheme: "bearer" }
      }
    },
    security: [{ bearer: [] }]
  }
);
try {
  if (result.error) throw new Error(result.error);
  console.log(result.api);
} finally {
  result.free();
}
```

Both arguments are required. The first must contain only the string `source`;
the second is a full OpenAPI 3.0 base document, not a partial patch or YAML text.
The required fields include `openapi`, `info.title`, `info.version`, and `paths`.
Malformed objects or OAL return the existing `{api: "", error: "..."}` result.
Inputs are serialized with JSON semantics and are not modified. Cyclic objects
and serialization errors produce diagnostics. Use ordinary JSON-compatible data,
not functions, BigInts, custom getters or `toJSON` implementations.

The existing CLI builder defines ownership: generated `paths` and
`components.schemas` **replace** those entire base collections. Other metadata,
document security, non-schema components and supported `x-` extensions are
retained through the typed OpenAPI model. This is not a deep merge and does not
preserve arbitrary unknown fields or original formatting; normal serialization
defaults may appear. A supplied base does not inherit the default title/server.
Reference resolution and complete OpenAPI semantic validation are not performed.

Rust callers retain `oal_wasm::compile(input: &str)` and can also call
`oal_wasm::compile_with_document(input: &str, base: oal_wasm::OpenAPI)`.
The CLI and its `--base` interface are unchanged. The new WASM entry point uses
the same builder without appending YAML fragments. Always deploy the generated
JavaScript bindings together with the matching WASM binary.
