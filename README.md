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
// Primitive types
let id1 = num;
```
```
// Records
let rec1 = {
  firstName str,
  lastName str
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
// Relations
let rel1 = uri1:get,put -> rec1;
```
```
// Merging records
let rec2 = rec1 & { age num };
```
```
// Typed alternative
let id2 = id1 | str;
```
```
// Untyped alternative
let any1 = id2 ~ rec2 ~ uri1;
```
```
// Function declaration
let f x y = rec2 & ( x | y );
```
```
// Function application
let rec3 = f { height num } { stuff any1 };
```
```
// Resources
res rel1;
res /something:get -> rec3;
```
```
/*
 * Block
 * comments
 */
```

## OpenAPI Output

The above source program generates the following OpenAPI specification:

```yaml
---
openapi: 3.0.1
info:
  title: Test OpenAPI specification
  version: 0.1.0
paths:
  "/some/path/{id}/template":
    get:
      responses:
        default:
          description: ""
          content:
            application/json:
              schema:
                type: object
                properties:
                  firstName:
                    type: string
                  lastName:
                    type: string
    put:
      responses:
        default:
          description: ""
          content:
            application/json:
              schema:
                type: object
                properties:
                  firstName:
                    type: string
                  lastName:
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
          description: ""
          content:
            application/json:
              schema:
                allOf:
                  - allOf:
                      - type: object
                        properties:
                          firstName:
                            type: string
                          lastName:
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
                                  - type: object
                                    properties:
                                      firstName:
                                        type: string
                                      lastName:
                                        type: string
                                  - type: object
                                    properties:
                                      age:
                                        type: number
                              - example: "/some/path/{id}/template"
                                type: string
                                format: uri-reference
```
