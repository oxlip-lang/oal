![Build](https://img.shields.io/github/actions/workflow/status/ebastien/openapi-lang/ci.yml?branch=master)
[![License](https://img.shields.io/badge/license-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

# An OpenAPI Language
OAL is a high-level functional programming language for designing
OpenAPI definitions.
As an [Interface Description Language](https://en.wikipedia.org/wiki/Interface_description_language), it is not general purpose and not Turing-complete, by design.
The motivation is to experiment with algebraic language abstractions over REST concepts,
not too dissimilar to [Sass/SCSS over CSS](https://sass-lang.com/),
and to consider OpenAPI as the assembly language of REST API design.

The language is strongly typed with global type inference.
The CLI generates OpenAPI 3.0.3 definitions in YAML format from the resources defined
in the source program.

## Capabilities

To address the challenges of handwriting OpenAPI definitions at scale, OAL offers three main capabilities:

- modular: project structure can be organized freely into folders and modules.
- composable: REST concepts are mapped to types and operations, enabling the composition of low-level values (e.g. JSON properties) into higher-level entities (e.g. HTTP end-points) as language expressions.
- functional: behavior can be encapsulated in functions and reused at will.

## Hello World

![Hello](images/hello.png)

## Installation
This step requires a [local Rust and Cargo installation](https://doc.rust-lang.org/cargo/getting-started/installation.html).

```
cargo install --path oal-client
```
Optional: a [VSCode language extension](https://github.com/ebastien/openapi-lang-vscode) is available for syntax highlighting and IDE capabilities.

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

## Examples of language constructs:
```
// Modules
use "some/other/module.oal";
```
```
// Primitives with inline annotations
let id1   = num `title: "some identifier"`;
let name  = str `pattern: "^[a-z]+$", example: sarah`;
let email = str `title: "E-mail address", format: email`;
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
  'firstName! name    `title: "First name"`
, 'lastName! name     `title: "Last name"`
, 'middleNames [name] `title: "Middle names"`
, 'email email
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
let rel1 = uri1 on op1, op2;
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
let etag = 'ETag! str `example: "675af34563dc-tr34"`;

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

res /something?{ 'q! str } on get : <headers={ifnmatch}> -> with_err @obj3;
```
```
/*
 * Block
 * comments
 */
```

[OpenAPI definition generated from this program](examples/openapi.yaml)

## Design decisions
- Rust got chosen both as a learning material and because of the maturity of its ecosystem for the development of compilers.
- An external domain specific language was preferred to minimize dependencies with language SDKs and runtimes for end-users.
- The parser emits a concrete syntax tree instead of an abstract syntax tree to enable interactive source code refactoring capabilities.

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
As a specialized language, OAL has the potential to provide a more compact syntax,
easier to learn, to read and to manage at scale.

- [ResponsibleAPI](https://github.com/responsibleapi/responsible)
