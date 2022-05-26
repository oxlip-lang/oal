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
// Importing modules
use "some/other/module.oal";
```
```
// Property types
# description: "some parameter"
let prop1 = 'id id1 `title: "some identifier"`;
let prop2 = 'n num;
let prop3 = 'age num;
```
```
// Objects
# description: "some stuff"
let obj1 = {
  'firstName str     `title: "First name"`
, 'lastName str      `title: "Last name"`
, 'middleNames [str] `title: "Middle names"`
};
```
```
// Templated URIs
let uri1 = /some/path/{ prop1 }/template;
```
```
// Undefined URIs
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

# summary: "does something else"
let op2 = get { 'q str } -> cnt1;
```
```
// Relations
let rel1 = uri1 ( op1, op2 );
```
```
// Joining schemas
let obj2 = obj1 & { prop3 };
```
```
// Typed alternative
let id2 = id1 | str;
```
```
// Untyped alternative
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
// Resources
res rel1;
res /something?{ 'q str } ( get -> obj3 );
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
      summary: does something else
      parameters:
        - in: query
          name: q
          required: true
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
                  lastName:
                    title: Last name
                    type: string
                  middleNames:
                    title: Middle names
                    type: array
                    items:
                      type: string
    put:
      summary: does something
      parameters:
        - in: query
          name: n
          required: true
          schema:
            type: number
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
                lastName:
                  title: Last name
                  type: string
                middleNames:
                  title: Middle names
                  type: array
                  items:
                    type: string
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
                  lastName:
                    title: Last name
                    type: string
                  middleNames:
                    title: Middle names
                    type: array
                    items:
                      type: string
    patch:
      summary: does something
      parameters:
        - in: query
          name: n
          required: true
          schema:
            type: number
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
                lastName:
                  title: Last name
                  type: string
                middleNames:
                  title: Middle names
                  type: array
                  items:
                    type: string
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
                  lastName:
                    title: Last name
                    type: string
                  middleNames:
                    title: Middle names
                    type: array
                    items:
                      type: string
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
          description: some other stuff
          content:
            application/json:
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
                          lastName:
                            title: Last name
                            type: string
                          middleNames:
                            title: Middle names
                            type: array
                            items:
                              type: string
                      - type: object
                        properties:
                          age:
                            type: number
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
                                      lastName:
                                        title: Last name
                                        type: string
                                      middleNames:
                                        title: Middle names
                                        type: array
                                        items:
                                          type: string
                                  - type: object
                                    properties:
                                      age:
                                        type: number
                              - example: "/some/path/{id}/template"
                                type: string
                                format: uri-reference
    parameters:
      - in: query
        name: q
        required: true
        schema:
          type: string
        style: form
```
</details>
