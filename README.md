![Build](https://img.shields.io/github/actions/workflow/status/ebastien/openapi-lang/ci.yml?branch=master)
[![License](https://img.shields.io/badge/license-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

# An OpenAPI Language
OAL is a high-level functional programming language for designing
OpenAPI definitions.
It is not a general purpose language. It is not Turing-complete, by design.
The motivation is to experiment with algebraic language abstractions over REST concepts,
not too dissimilar to [Sass/SCSS over CSS](https://sass-lang.com/).
The ambition of the author is to consider OpenAPI as the assembly language of REST API design.

The language is strongly typed with global type inference.
Due to the experimental nature of this project, error handling is rudimentary.
The CLI generates OpenAPI 3.0.3 definitions in YAML format from the resources defined
in the source program.

## Installation
This step requires a [local Rust and Cargo installation](https://doc.rust-lang.org/cargo/getting-started/installation.html).

```
cargo install --path oal-cli
```

## Usage
```
    oal-cli [OPTIONS] --input <INPUT> --output <OUTPUT>

OPTIONS:
    -b, --base <BASE>        The path to a base OpenAPI description
    -h, --help               Print help information
    -i, --input <INPUT>      The path to the source program
    -o, --output <OUTPUT>    The path to the output OpenAPI description
```

### Compiling the example program
```
oal-cli --base examples/base.yaml --input examples/main.oal --output examples/openapi.yaml
```

## Examples of language constructs:
```
// Modules
use "some/other/module.oal";
```
```
// Primitives with inline annotations
let id1 = num  `title: "some identifier"`;
let name = str `pattern: "^[a-z]+$", example: sarah`;
```
```
// Properties with both statement and inline annotations
# description: "some parameter"
let prop1 = 'id id1;

let prop2 = 'n num   `minimum: 0, maximum: 99.99, example: 42`;
let prop3 = 'age int `minimum: 0, maximum: 999`;
```
```
// Objects
// The at-sign prefix denotes a reference variable whose value
// is registered as a component in the OpenAPI output.
# description: "some stuff"
let @obj1 = {
  'firstName name     `title: "First name", required: true`
, 'lastName name      `title: "Last name", required: true`
, 'middleNames [name] `title: "Middle names"`
};
```
```
// Templated URIs
let uri1 = /some/path/{ prop1 }/template;
```
```
// Unspecified URIs
let uri2 = uri;
```
```
// Contents
# description: "some content"
# examples: { default: "examples/stuff.json" }
let cnt1 = <@obj1>;
```
```
// Operations
# summary: "does something"
let op1 = patch, put { prop2 } : cnt1 -> cnt1;

# summary: "does something else", tags: [blah]
let op2 = get { 'q str } -> cnt1;
```
```
// Relations
let rel1 = uri1 ( op1, op2 );
```
```
// Combining schemas
let @obj2 = @obj1 & { prop3 };
```
```
// Typed schema alternative
let id2 = id1 | str;
```
```
// Untyped schema alternative
let any1 = id2 ~ @obj2 ~ uri1;
```
```
// Function declaration
let f x y = @obj2 & ( x | y );
```
```
// Function application
# description: "some other stuff"
# examples: { default: "examples/other_stuff.json" }
let @obj3 = f { 'height num } { 'stuff any1 };
```
```
// Headers
# description: "identifier for a specific version of a resource"
# required: true
let etag = 'ETag str `example: "675af34563dc-tr34"`;

# description: "makes the request conditional"
let ifnmatch = 'If-None-Match str;
```
```
// Media types
let vendor = "application/vnd.blah+json";
let problem = "application/problem+json";
```
```
// Combining contents into ranges
let with_err s = <status=200, media=vendor, headers={etag}, s>  `description: "all good"`
              :: <status=5XX, media=problem, {}>                `description: "internal error"`
              :: <status=4XX, media=problem, {}>                `description: "bad request"`
              :: <>                                             `description: "no content"`;
```
```
// Binding everything together as resources
res rel1;

res /something?{ 'q str } (
  get : <headers={ifnmatch}> -> with_err @obj3
);
```
```
/*
 * Block
 * comments
 */
```

[OpenAPI definition generated from this program](examples/openapi.yaml)

## Related work and comparison with OAL
- [Cadl](https://github.com/microsoft/cadl)
- [Smithy IDL](https://github.com/awslabs/smithy)

Arguably, Cadl and Smithy provide an opinionated service interface abstraction
that happens to compile down to an OpenAPI definition.
As they take on a broader scope than REST APIs (e.g. GraphQL, gRPC),
it is not their objective to behave as a preprocessor and to offer feature parity with OpenAPI.
The language design philosophy for both Cald and Smithy looks similar and
influenced by familiar object-oriented and general purpose languages.
Cadl's support for parameterized data types (aka. templates) makes it more programmable than Smithy.

- [KCL](https://github.com/KusionStack/KCLVM)
- [CUE](https://github.com/cue-lang/cue)
- [Dhall](https://github.com/dhall-lang/dhall-lang)

KCL, CUE and Dhall belong to a different category of languages focusing on programmable configuration.
Their domain of concern is the management and validation of configuration files at scale,
e.g. JSON or YAML files for large Kubernetes deployments.
The approach is to define a higher-level language with functions, types, modules and data constraints,
able to compile down to the target configuration format.
It obviously applies to the generation of OpenAPI definitions by the very nature of the OpenAPI specification being a JSON-based format.
That being said, REST concepts are not first-class citizen for those languages.
The domain of programmability only applies to the generation of JSON/YAML objects, lists and scalar values.
It does not directly model the composition of REST entities as a service interface definition.

OAL takes a different approach by defining an algebra and a functional evaluation strategy
dedicated to the composition of low-level REST concepts into modular OpenAPI definitions.
One can argue that extensible languages like Dhall could achieve similar objectives.
The opinion of the author of OAL is that a specialized language would provide a more compact syntax,
easier to learn, to read and to manage at scale.
