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
oal-cli -b examples/base.yaml -i examples/main.oal -o examples/openapi.yaml
```

## Examples of language constructs:

<div>
    <style>
        p.p1 {margin: 0; line-height: 16px; font: 12px Menlo; color: #598a43; background-color: #000000}
        p.p2 {margin: 0; line-height: 16px; font: 12px Menlo; color: #c27e65; background-color: #000000}
        p.p3 {margin: 0; line-height: 16px; font: 12px Menlo; color: #cacaca; background-color: #000000; min-height: 19.0px}
        p.p4 {margin: 0; line-height: 16px; font: 12px Menlo; color: #8cd3fe; background-color: #000000}
        p.p5 {margin: 0; line-height: 16px; font: 12px Menlo; color: #cacaca; background-color: #000000}
        span.s1 {font-kerning: none}
        span.s2 {font-kerning: none; color: #b76fb3; -webkit-text-stroke: 0 #b76fb3}
        span.s3 {font-kerning: none; color: #cacaca; -webkit-text-stroke: 0 #cacaca}
        span.s4 {font-kerning: none; color: #8cd3fe; -webkit-text-stroke: 0 #8cd3fe}
        span.s5 {font-kerning: none; color: #4689cc; -webkit-text-stroke: 0 #4689cc}
        span.s6 {font-kerning: none; color: #a7c598; -webkit-text-stroke: 0 #a7c598}
        span.s7 {font-kerning: none; color: #c27e65; -webkit-text-stroke: 0 #c27e65}
    </style>
    <p class="p1"><span class="s1">// Modules</span></p>
    <p class="p2"><span class="s2">use</span><span class="s3"> </span><span class="s1">"some/other/module.oal"</span><span class="s3">;</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p1"><span class="s1">// Primitives with inline annotations</span></p>
    <p class="p2"><span class="s2">let</span><span class="s3"> </span><span class="s4">id1</span><span class="s3"> = </span><span class="s5">num</span><span class="s3"><span class="Apple-converted-space">  </span></span><span class="s1">`title: "some identifier"`</span><span class="s3">;</span></p>
    <p class="p2"><span class="s2">let</span><span class="s3"> </span><span class="s4">name</span><span class="s3"> = </span><span class="s5">str</span><span class="s3"> </span><span class="s1">`pattern: "^[a-z]+$"`</span><span class="s3">;</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p1"><span class="s1">// Properties with both statement and inline annotations</span></p>
    <p class="p2"><span class="s1"># description: "some parameter"</span></p>
    <p class="p4"><span class="s2">let</span><span class="s3"> </span><span class="s1">prop1</span><span class="s3"> = </span><span class="s1">'id</span><span class="s3"> </span><span class="s1">id1</span><span class="s3">;</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p2"><span class="s2">let</span><span class="s3"> </span><span class="s4">prop2</span><span class="s3"> = </span><span class="s4">'n</span><span class="s3"> </span><span class="s5">num</span><span class="s3"> <span class="Apple-converted-space">  </span></span><span class="s1">`minimum: 0, maximum: 99.99`</span><span class="s3">;</span></p>
    <p class="p2"><span class="s2">let</span><span class="s3"> </span><span class="s4">prop3</span><span class="s3"> = </span><span class="s4">'age</span><span class="s3"> </span><span class="s5">int</span><span class="s3"> </span><span class="s1">`minimum: 0, maximum: 999`</span><span class="s3">;</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p1"><span class="s1">// Objects</span></p>
    <p class="p2"><span class="s1"># description: "some stuff"</span></p>
    <p class="p5"><span class="s2">let</span><span class="s1"> </span><span class="s4">obj1</span><span class="s1"> = {</span></p>
    <p class="p2"><span class="s3"><span class="Apple-converted-space">  </span></span><span class="s4">'firstName</span><span class="s3"> </span><span class="s4">name</span><span class="s3"> <span class="Apple-converted-space">    </span></span><span class="s1">`title: "First name", required: true`</span></p>
    <p class="p2"><span class="s3">, </span><span class="s4">'lastName</span><span class="s3"> </span><span class="s4">name</span><span class="s3"><span class="Apple-converted-space">      </span></span><span class="s1">`title: "Last name", required: true`</span></p>
    <p class="p2"><span class="s3">, </span><span class="s4">'middleNames</span><span class="s3"> [</span><span class="s4">name</span><span class="s3">] </span><span class="s1">`title: "Middle names"`</span></p>
    <p class="p5"><span class="s1">};</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p1"><span class="s1">// Templated URIs</span></p>
    <p class="p2"><span class="s2">let</span><span class="s3"> </span><span class="s4">uri1</span><span class="s3"> = </span><span class="s1">/some/path/</span><span class="s3">{ </span><span class="s4">prop1</span><span class="s3"> }</span><span class="s1">/template</span><span class="s3">;</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p1"><span class="s1">// Undefined URIs</span></p>
    <p class="p5"><span class="s2">let</span><span class="s1"> </span><span class="s4">uri2</span><span class="s1"> = </span><span class="s5">uri</span><span class="s1">;</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p1"><span class="s1">// Contents</span></p>
    <p class="p2"><span class="s1"># description: "some content"</span></p>
    <p class="p4"><span class="s2">let</span><span class="s3"> </span><span class="s1">cnt1</span><span class="s3"> = &lt;</span><span class="s1">obj1</span><span class="s3">&gt;;</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p1"><span class="s1">// Operations</span></p>
    <p class="p2"><span class="s1"># summary: "does something"</span></p>
    <p class="p5"><span class="s2">let</span><span class="s1"> </span><span class="s4">op1</span><span class="s1"> = </span><span class="s5">patch</span><span class="s1">, </span><span class="s5">put</span><span class="s1"> { </span><span class="s4">prop2</span><span class="s1"> } : </span><span class="s4">cnt1</span><span class="s1"> -&gt; </span><span class="s4">cnt1</span><span class="s1">;</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p2"><span class="s1"># summary: "does something else", tags: [blah]</span></p>
    <p class="p5"><span class="s2">let</span><span class="s1"> </span><span class="s4">op2</span><span class="s1"> = </span><span class="s5">get</span><span class="s1"> { </span><span class="s4">'q</span><span class="s1"> </span><span class="s5">str</span><span class="s1"> } -&gt; </span><span class="s4">cnt1</span><span class="s1">;</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p1"><span class="s1">// Relations</span></p>
    <p class="p4"><span class="s2">let</span><span class="s3"> </span><span class="s1">rel1</span><span class="s3"> = </span><span class="s1">uri1</span><span class="s3"> ( </span><span class="s1">op1</span><span class="s3">, </span><span class="s1">op2</span><span class="s3"> );</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p1"><span class="s1">// Combining schemas</span></p>
    <p class="p4"><span class="s2">let</span><span class="s3"> </span><span class="s1">obj2</span><span class="s3"> = </span><span class="s1">obj1</span><span class="s3"> &amp; { </span><span class="s1">prop3</span><span class="s3"> };</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p1"><span class="s1">// Typed schema alternative</span></p>
    <p class="p5"><span class="s2">let</span><span class="s1"> </span><span class="s4">id2</span><span class="s1"> = </span><span class="s4">id1</span><span class="s1"> | </span><span class="s5">str</span><span class="s1">;</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p1"><span class="s1">// Untyped schema alternative</span></p>
    <p class="p4"><span class="s2">let</span><span class="s3"> </span><span class="s1">any1</span><span class="s3"> = </span><span class="s1">id2</span><span class="s3"> ~ </span><span class="s1">obj2</span><span class="s3"> ~ </span><span class="s1">uri1</span><span class="s3">;</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p1"><span class="s1">// Function declaration</span></p>
    <p class="p5"><span class="s2">let</span><span class="s1"> </span><span class="s4">f</span><span class="s1"> </span><span class="s4">x</span><span class="s1"> </span><span class="s4">y</span><span class="s1"> = </span><span class="s4">obj2</span><span class="s1"> &amp; ( </span><span class="s4">x</span><span class="s1"> | </span><span class="s4">y</span><span class="s1"> );</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p1"><span class="s1">// Function application</span></p>
    <p class="p2"><span class="s1"># description: "some other stuff"</span></p>
    <p class="p4"><span class="s2">let</span><span class="s3"> </span><span class="s1">obj3</span><span class="s3"> = </span><span class="s1">f</span><span class="s3"> { </span><span class="s1">'height</span><span class="s3"> </span><span class="s5">num</span><span class="s3"> } { </span><span class="s1">'stuff</span><span class="s3"> </span><span class="s1">any1</span><span class="s3"> };</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p1"><span class="s1">// Headers</span></p>
    <p class="p2"><span class="s1"># description: "identifier for a specific version of a resource"</span></p>
    <p class="p4"><span class="s2">let</span><span class="s3"> </span><span class="s1">etag</span><span class="s3"> = </span><span class="s1">'ETag</span><span class="s3"> </span><span class="s5">str</span><span class="s3">;</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p2"><span class="s1"># description: "makes the request conditional"</span></p>
    <p class="p4"><span class="s2">let</span><span class="s3"> </span><span class="s1">ifnmatch</span><span class="s3"> = </span><span class="s1">'If-None-Match</span><span class="s3"> </span><span class="s5">str</span><span class="s3">;</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p1"><span class="s1">// Media types</span></p>
    <p class="p2"><span class="s2">let</span><span class="s3"> </span><span class="s4">vendor</span><span class="s3"> = </span><span class="s1">"application/vnd.blah+json"</span><span class="s3">;</span></p>
    <p class="p2"><span class="s2">let</span><span class="s3"> </span><span class="s4">problem</span><span class="s3"> = </span><span class="s1">"application/problem+json"</span><span class="s3">;</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p1"><span class="s1">// Combining contents into ranges</span></p>
    <p class="p2"><span class="s2">let</span><span class="s3"> </span><span class="s4">with_err</span><span class="s3"> </span><span class="s4">s</span><span class="s3"> = &lt;</span><span class="s5">status</span><span class="s3">=</span><span class="s6">200</span><span class="s3">, </span><span class="s5">media</span><span class="s3">=</span><span class="s4">vendor</span><span class="s3">, </span><span class="s5">headers</span><span class="s3">={</span><span class="s4">etag</span><span class="s3">}, </span><span class="s4">s</span><span class="s3">&gt;<span class="Apple-converted-space">  </span></span><span class="s1">`description: "all good"`</span></p>
    <p class="p5"><span class="s1"><span class="Apple-converted-space">              </span>:: &lt;</span><span class="s5">status</span><span class="s1">=5XX, </span><span class="s5">media</span><span class="s1">=</span><span class="s4">problem</span><span class="s1">, {}&gt;<span class="Apple-converted-space">                </span></span><span class="s7">`description: "internal error"`</span></p>
    <p class="p5"><span class="s1"><span class="Apple-converted-space">              </span>:: &lt;</span><span class="s5">status</span><span class="s1">=4XX, </span><span class="s5">media</span><span class="s1">=</span><span class="s4">problem</span><span class="s1">, {}&gt;<span class="Apple-converted-space">                </span></span><span class="s7">`description: "bad request"`</span></p>
    <p class="p5"><span class="s1"><span class="Apple-converted-space">              </span>:: &lt;&gt; <span class="Apple-converted-space">                                            </span></span><span class="s7">`description: "no content"`</span><span class="s1">;</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p1"><span class="s1">// Binding everything together as resources</span></p>
    <p class="p4"><span class="s2">res</span><span class="s3"> </span><span class="s1">rel1</span><span class="s3">;</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p2"><span class="s2">res</span><span class="s3"> </span><span class="s1">/something</span><span class="s3">?{ </span><span class="s4">'q</span><span class="s3"> </span><span class="s5">str</span><span class="s3"> } (</span></p>
    <p class="p4"><span class="s3"><span class="Apple-converted-space">  </span></span><span class="s5">get</span><span class="s3"> : &lt;</span><span class="s5">headers</span><span class="s3">={</span><span class="s1">ifnmatch</span><span class="s3">},&gt; -&gt; </span><span class="s1">with_err</span><span class="s3"> </span><span class="s1">obj3</span></p>
    <p class="p5"><span class="s1">);</span></p>
    <p class="p3"><span class="s1"></span><br></p>
    <p class="p1"><span class="s1">/*</span></p>
    <p class="p1"><span class="s1"><span class="Apple-converted-space"> </span>* Block</span></p>
    <p class="p1"><span class="s1"><span class="Apple-converted-space"> </span>* comments</span></p>
    <p class="p1"><span class="s1"><span class="Apple-converted-space"> </span>*/</span></p>
    <p class="p3"><span class="s1"></span><br></p>
</div>

[OpenAPI definition generated from this program](examples/openapi.yaml)
