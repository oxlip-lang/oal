![Build](https://img.shields.io/github/workflow/status/ebastien/openapi-lang/ci)
[![License](https://img.shields.io/badge/license-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

# OpenAPI Language

An experiment on a high-level functional programming language for designing
OpenAPI specifications.
This is not a general purpose language.
The motivation is to play with algebraic language abstractions on top of OpenAPI
in a similar fashion as Sass/SCSS over CSS.
The ambition of the author is to consider OpenAPI as the assembly language of API design. 

The language is statically typed with global type inference.
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
oal-cli -b examples/base.yaml -i examples/main.oal -o openapi.yaml
```

## Examples of language constructs:
```
// Modules
use "some/other/module.oal";
```
```
// Primitives with inline annotations
let id1 = num  `title: "some identifier"`;
let name = str `pattern: "^[a-z]+$"`;
```
```
// Properties with both statement and inline annotations
# description: "some parameter"
let prop1 = 'id id1;

let prop2 = 'n num   `minimum: 0, maximum: 99.99`;
let prop3 = 'age int `minimum: 0, maximum: 999`;
```
```
// Objects
# description: "some stuff"
let obj1 = {
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
let cnt1 = <obj1>;
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
let obj2 = obj1 & { prop3 };
```
```
// Typed schema alternative
let id2 = id1 | str;
```
```
// Untyped schema alternative
let any1 = id2 ~ obj2 ~ uri1;
```
```
// Function declaration
let f x y = obj2 & ( x | y );
```
```
// Function application
# description: "some other stuff"
let obj3 = f { 'height num } { 'stuff any1 };
```
```
// Combining contents into ranges
let with_err c = c :: <application/problem+json,500,{}> `description: "went wrong"`;
```
```
// Binding everything together as resources
res rel1;
res /something?{ 'q str } (
  get -> with_err <application/vnd.blah+json,200,obj3> `description: "all good"`
      :: <>                                            `description: "no content"`
);
```
```
/*
 * Block
 * comments
 */
```

<details>
  <summary>OpenAPI definition generated from the program above</summary>

```yaml
---
openapi: 3.0.3
info:
  title: Example
  description: Example
  version: 1.0.0
servers:
  - url: /
paths:
  "/some/path/{id}/template":
    get:
      tags:
        - blah
      summary: does something else
      parameters:
        - in: query
          name: q
          schema:
            type: string
          style: form
      responses:
        default:
          description: some content
          content:
            application/json:
              schema:
                description: some stuff
                type: object
                properties:
                  firstName:
                    title: First name
                    type: string
                    pattern: "^[a-z]+$"
                  lastName:
                    title: Last name
                    type: string
                    pattern: "^[a-z]+$"
                  middleNames:
                    title: Middle names
                    type: array
                    items:
                      type: string
                      pattern: "^[a-z]+$"
                required:
                  - firstName
                  - lastName
    put:
      summary: does something
      parameters:
        - in: query
          name: n
          schema:
            type: number
            minimum: 0.0
            maximum: 99.99
          style: form
      requestBody:
        description: some content
        content:
          application/json:
            schema:
              description: some stuff
              type: object
              properties:
                firstName:
                  title: First name
                  type: string
                  pattern: "^[a-z]+$"
                lastName:
                  title: Last name
                  type: string
                  pattern: "^[a-z]+$"
                middleNames:
                  title: Middle names
                  type: array
                  items:
                    type: string
                    pattern: "^[a-z]+$"
              required:
                - firstName
                - lastName
      responses:
        default:
          description: some content
          content:
            application/json:
              schema:
                description: some stuff
                type: object
                properties:
                  firstName:
                    title: First name
                    type: string
                    pattern: "^[a-z]+$"
                  lastName:
                    title: Last name
                    type: string
                    pattern: "^[a-z]+$"
                  middleNames:
                    title: Middle names
                    type: array
                    items:
                      type: string
                      pattern: "^[a-z]+$"
                required:
                  - firstName
                  - lastName
    patch:
      summary: does something
      parameters:
        - in: query
          name: n
          schema:
            type: number
            minimum: 0.0
            maximum: 99.99
          style: form
      requestBody:
        description: some content
        content:
          application/json:
            schema:
              description: some stuff
              type: object
              properties:
                firstName:
                  title: First name
                  type: string
                  pattern: "^[a-z]+$"
                lastName:
                  title: Last name
                  type: string
                  pattern: "^[a-z]+$"
                middleNames:
                  title: Middle names
                  type: array
                  items:
                    type: string
                    pattern: "^[a-z]+$"
              required:
                - firstName
                - lastName
      responses:
        default:
          description: some content
          content:
            application/json:
              schema:
                description: some stuff
                type: object
                properties:
                  firstName:
                    title: First name
                    type: string
                    pattern: "^[a-z]+$"
                  lastName:
                    title: Last name
                    type: string
                    pattern: "^[a-z]+$"
                  middleNames:
                    title: Middle names
                    type: array
                    items:
                      type: string
                      pattern: "^[a-z]+$"
                required:
                  - firstName
                  - lastName
    parameters:
      - in: path
        name: id
        description: some parameter
        required: true
        schema:
          title: some identifier
          type: number
        style: simple
  /something:
    get:
      responses:
        default:
          description: no content
        "200":
          description: all good
          content:
            application/vnd.blah+json:
              schema:
                description: some other stuff
                allOf:
                  - allOf:
                      - description: some stuff
                        type: object
                        properties:
                          firstName:
                            title: First name
                            type: string
                            pattern: "^[a-z]+$"
                          lastName:
                            title: Last name
                            type: string
                            pattern: "^[a-z]+$"
                          middleNames:
                            title: Middle names
                            type: array
                            items:
                              type: string
                              pattern: "^[a-z]+$"
                        required:
                          - firstName
                          - lastName
                      - type: object
                        properties:
                          age:
                            type: integer
                            minimum: 0
                            maximum: 999
                  - oneOf:
                      - type: object
                        properties:
                          height:
                            type: number
                      - type: object
                        properties:
                          stuff:
                            anyOf:
                              - oneOf:
                                  - type: number
                                  - type: string
                              - allOf:
                                  - description: some stuff
                                    type: object
                                    properties:
                                      firstName:
                                        title: First name
                                        type: string
                                        pattern: "^[a-z]+$"
                                      lastName:
                                        title: Last name
                                        type: string
                                        pattern: "^[a-z]+$"
                                      middleNames:
                                        title: Middle names
                                        type: array
                                        items:
                                          type: string
                                          pattern: "^[a-z]+$"
                                    required:
                                      - firstName
                                      - lastName
                                  - type: object
                                    properties:
                                      age:
                                        type: integer
                                        minimum: 0
                                        maximum: 999
                              - example: "/some/path/{id}/template"
                                type: string
                                format: uri-reference
        "500":
          description: went wrong
          content:
            application/problem+json:
              schema:
                type: object
    parameters:
      - in: query
        name: q
        schema:
          type: string
        style: form
```
</details>
