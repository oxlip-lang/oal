# OpenAPI Language

An experiment on a high-level functional programming language for designing OpenAPI specifications.
This is not a general purpose language.
The motivation is to play with language abstractions on top of OpenAPI in a similar fashion as Sass over CSS.
The ambition of the author is to consider OpenAPI as the assembly language of API design. 

The language is statically typed with global type inference.
Due to the experimental nature of this project, error handling is rudimentary.
The main compiler program takes two arguments: a source and target file names.
The output is an OpenAPI 3.0.1 specification in YAML format,
compiled from the resources defined in the source program.

## Examples of language constructs:
```
// Importing modules
use "some/other/module.oal";
```
```
// Primitive types
let id1 = num;
```
```
// Objects
# description: "some stuff"
let rec1 = {
  firstName str     `title: "First name"`
, lastName str      `title: "Last name"`
, middleNames [str] `title: "Middle names"`
};
```
```
// Templated URIs
let uri1 = /some/path/{ id id1 }/template;
```
```
// Undefined URIs
let uri2 = uri;
```
```
// Contents
# description: "some content"
let cnt1 = <rec1>;
```
```
// Operations
# summary: "does something"
let op1 = patch, put : cnt1 -> cnt1;

# summary: "does something else"
let op2 = get -> cnt1;
```
```
// Relations
let rel1 = uri1 ( op1, op2 );
```
```
// Joining schemas (allOf)
let rec2 = rec1 & { age num };
```
```
// Typed alternative (oneOf)
let id2 = id1 | str;
```
```
// Untyped alternative (anyOf)
let any1 = id2 ~ rec2 ~ uri1;
```
```
// Function declaration
let f x y = rec2 & ( x | y );
```
```
// Function application
# description: "some other stuff"
let rec3 = f { height num } { stuff any1 };
```
```
// Resources
res rel1;
res /something ( get -> rec3 );
```
```
/*
 * Block
 * comments
 */
```

<details>
  <summary>Example of OpenAPI specification generated from the program above</summary>

```yaml
---
openapi: 3.0.1
info:
  title: Test OpenAPI specification
  version: 0.1.0
servers:
  - url: /
paths:
  "/some/path/{id}/template":
    get:
      summary: does something else
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
        required: true
        schema:
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
```
</details>
