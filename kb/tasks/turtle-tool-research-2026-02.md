# Turtle Tool Research: Linting, Formatting, and Validation

## Date: 2026-02-08

## Overview

Research into popular Turtle (TTL/RDF) linters, formatters, and validators to identify features that should inform our Biome Turtle implementation. 20+ tools surveyed across Java, Rust, Python, Node.js, .NET, PHP, Ruby, and C ecosystems.

---

## Tools Surveyed

### 1. rapper (Raptor RDF Library)

**Ecosystem:** C library, command-line tool

**Linting/Checks:**
- Strict vs. lax parsing modes (`rdf:bagID` warning in lax, error in strict)
- Warning/error message control (`-w` ignore warnings, `-e` ignore errors)
- Triple counting mode (`-c`)
- Namespace tracking and URI tracing

**Formatting:**
- Round-trip formatting via `rapper -i turtle -o turtle`
- Configurable base URI for input (`-I`) and output (`-O`)
- Namespace/prefix injection via `-f` option

**Validation:**
- Full W3C Turtle syntax validation
- IRI parsing and validation
- Configurable parser features via `-f FEATURE=VALUE`

---

### 2. RIOT (Apache Jena)

**Ecosystem:** Java, command-line tool

**Linting/Checks:**
- `--validate` mode (equivalent to `--strict --sink --check=true`)
- **IRI validation via `CheckerIRI`:**
  - Illegal characters (spaces, tabs, control characters, angle brackets)
  - "Bad IRI" errors vs. "Unwise IRI" warnings
  - Scheme-specific violations (reported as warnings)
  - Relative IRI resolution checking
- **Literal validation via `CheckerLiteral`:**
  - Language tag validation (BCP47 pattern: `[a-zA-Z]{1,8}(-[a-zA-Z0-9]{1,8})*`)
  - Datatype checking
  - Carriage return detection in single-quoted literals
- Subject/predicate/object position validation (literals cannot appear as subjects)

**Formatting:**
- **Three format variants:**
  - `TURTLE_PRETTY`: Pretty-printed with subject grouping, sorted predicates
  - `TURTLE_BLOCKS`: Streamed block format with partial subject grouping
  - `TURTLE_FLAT`: One triple per line, minimal memory
- **Indentation styles:**
  - `ttl:indentStyle=wide` (default): Aligns predicates and objects
  - `ttl:indentStyle=long`: Narrower indentation
- **Directive styles:**
  - `ttl:directiveStyle=sparql/rdf11`: Uses `PREFIX` and `BASE`
  - `ttl:directiveStyle=at/rdf10`: Uses `@prefix` and `@base`
- `ttl:omitBase`: Controls whether base URI appears in output

**Validation:**
- Complete W3C Turtle 1.1 syntax validation
- Standalone `iri` command-line tool for IRI parsing/error reporting
- Strict vs. lenient parsing modes

---

### 3. turtle-formatter (atextor)

**Ecosystem:** Java library

**Formatting (25+ configuration options):**

| Option | Description | Default |
|--------|-------------|---------|
| `alignPrefixes` | Align prefix declarations on colons | `false` |
| `alignPredicates` | Align predicates vertically | `false` |
| `alignObjects` | Align objects vertically | `false` |
| `firstPredicateInNewLine` | First predicate on new line | `false` |
| `indentStyle` | `SPACE` or `TAB` | `SPACE` |
| `indentSize` | Spaces per indent level | `2` |
| `endOfLine` | `LF`, `CR`, or `CRLF` | `LF` |
| `insertFinalNewLine` | Add newline after final statement | `true` |
| `charset` | Output encoding | `UTF_8` |
| `useAForRdfType` | Write `rdf:type` as `a` | `true` |
| `keepUnusedPrefixes` | Retain unused prefixes | `false` |
| `useCommaByDefault` | Commas for same predicates | `false` |
| `commaForPredicate` | Specific predicates always get commas | `{rdf:type}` |
| `noCommaForPredicate` | Specific predicates never get commas | `{}` |
| `quoteStyle` | `ALWAYS_SINGLE_QUOTES`, `TRIPLE_QUOTES_FOR_MULTILINE`, `ALWAYS_TRIPLE_QUOTES` | `TRIPLE_QUOTES_FOR_MULTILINE` |
| `prefixOrder` | Order of `@prefix` directives | `rdf, rdfs, xsd, owl` first |
| `subjectOrder` | Order of subjects by `rdf:type` | OWL/RDF classes |
| `predicateOrder` | Predicate ordering | `rdf:type, rdfs:label, rdfs:comment, dcterms:description` first |
| `objectOrder` | Object ordering | OWL property types |
| `wrapListItems` | RDF list line breaks | `FOR_LONG_LINES` |
| `normalizeLineTerminators` | Normalize multiline string endings | `true` |
| `doubleFormat` | Number format for `xsd:double` | `0.####E0` |
| `anonymousNodeIdGenerator` | Blank node naming function | `(r, i) -> "gen" + i` |
| `emptyRdfBase` | Base URI to omit | `urn:turtleformatter:internal` |

---

### 4. turtlefmt (helsing-ai)

**Ecosystem:** Rust

**Formatting:**
- Validates file syntax
- Consistent indentation and line breaks
- Normalizes string and IRI escape sequences (minimizes escaping)
- Enforces `"` instead of `'` in literals
- Uses short notation for booleans, integers, decimals, and doubles
- Writes `rdf:type` as `a`
- Check mode with patch output (`--check`)
- Directory-recursive formatting

**Note:** Still in development, format not yet stable. Opinionated, zero-configuration design.

---

### 5. prttl

**Ecosystem:** Rust

**Formatting:**
- **Diff-optimized output** with extensive newlines (designed for Git workflows)
- Configurable indentation (`--indentation <NUM>`, default: 2)
- `--single-leafed-new-lines`: Lone predicate-object pairs on new lines
- `--no-sparql-syntax`: Use `@base`/`@prefix` instead of `BASE`/`PREFIX`
- `--canonicalize`: W3C RDF canonicalization of blank nodes
- `--label-all-blank-nodes`: Labels for all blank nodes instead of nesting
- **Predicate ordering:** `--pred-order` with custom list or `--pred-order-preset` (owl, skos, shacl, shex, rdf)
- **Subject type ordering:** `--subj-type-order` with custom list or `--subj-type-order-preset`
- `--no-prtr-sorting`: Disable `prtr:sortingId`-based sorting
- Normalizes string and IRI escape sequences
- Converts single quotes to double quotes
- Uses short notation for booleans, integers, decimals, doubles
- Replaces `rdf:type` with `a`
- **Removes all Turtle syntax comments**

---

### 6. rdflint

**Ecosystem:** Java (Apache Jena-based), VS Code extension available

**Linting/Checks:**
- Syntax checking of RDF and Turtle files
- **Undefined subject detection**: Flags subjects referenced as predicates/objects that are never defined
- **Custom SPARQL-based checks**: Users write SPARQL queries as custom validation rules
- **Degrade validation**: Detects data quality regression across versions
- **Datatype and outlier validation**: Validates values conform to declared datatypes, detects statistical outliers
- **Literal trim validation**: Detects improper leading/trailing whitespace in literal values
- **File encoding validation**: Verifies correct character encoding

**Validation:**
- **SHACL constraint validation**: Validates data against SHACL shapes
- Full syntax validation via underlying Jena parser
- Interactive SPARQL playground

---

### 7. turtle-validator (npm / IDLabResearch)

**Ecosystem:** Node.js/npm (uses N3.js)

**Validation:**
- Full Turtle syntax validation
- Full N-Triples syntax validation
- XSD datatype error detection
- URL-based validation (can fetch and validate remote files)
- Browser-based validation interface

---

### 8. N3.js

**Ecosystem:** Node.js/npm

**Features:**
- Streaming parser for Turtle, TriG, N-Triples, N-Quads, N3
- Strict mode via `format` option for fault-intolerant behavior
- Streaming writer with prefix declaration support
- RDF 1.2 support, RDFJS spec-compliant

---

### 9. rdf-validate-shacl (npm)

**Ecosystem:** Node.js/npm, pure JavaScript

**SHACL Constraint Types:**
- **Cardinality:** `sh:minCount`, `sh:maxCount`
- **Type:** `sh:datatype`, `sh:class`, `sh:nodeKind`
- **String:** `sh:pattern`, `sh:minLength`, `sh:maxLength`
- **Range:** `sh:minInclusive`, `sh:minExclusive`, `sh:maxInclusive`, `sh:maxExclusive`
- **Value enumeration:** `sh:in`
- **Logical combinations:** `sh:and`, `sh:or`, `sh:not`, `sh:xone`
- Returns `ValidationReport` with conformance, results, severity, message, path, focus node

---

### 10. pySHACL (Python)

**Ecosystem:** Python (built on rdflib)

**Validation:**
- Full W3C SHACL specification support
- SHACL-AF Rules (inferencing)
- SPARQL Remote Graph Mode (read-only remote validation)
- OWL2 RL Profile-based expansion
- Targeted focus node and shape-selective validation

---

### 11. Stardog Tools

**11a. Stardog ICV (Integrity Constraint Validation):**
- SHACL-based constraint validation
- OWL, SWRL, and SPARQL constraint languages
- Guard mode: transactions violating constraints are rejected
- Named graph-scoped validation

**11b. Stardog Turtle Language Server:**
- Syntax diagnostics for W3C Turtle via LSP
- Error recovery with "expected symbols" hints
- Also supports SPARQL, TriG, SHACL, SMS, SRS, GraphQL

---

### 12. rdflib (Python)

**Ecosystem:** Python

**Formatting:**
- **Standard Turtle** (`format='turtle'`):
  - Predicate ordering: `rdf:type` first, then `rdfs:label`, then others
  - Subject grouping, blank node compression, collection notation
  - `canonicalize` and `validate` options
- **LongTurtle** (`format='longturtle'`):
  - Uses `PREFIX`/`BASE` instead of `@prefix`/`@base`
  - Newline after `rdf:type`/`a`
  - Newline + indent for all triples with more than one object
  - **Designed specifically for Git diff optimization**

**Validation:**
- Full parsing with error reporting
- `validate` option on reader and writer

---

### 13. otsrdflib (Ordered Turtle Serializer for rdflib)

**Ecosystem:** Python (rdflib plugin)

**Formatting:**
- Class-based ordering via `class_order` attribute
- Custom sort key generators via `sorters` attribute (regex-based URI matching)
- Per-class sorting via `sorters_by_class` dictionary
- Namespace management with duplicate-binding detection
- Default alphabetical URI sorting
- Deterministic, reproducible output (excellent for version control)

---

### 14. EasyRdf

**Ecosystem:** PHP

**Formatting:** Turtle serializer with QName output
**Validation:** Syntax validation during parsing with line/column error reporting

---

### 15. dotNetRDF

**Ecosystem:** .NET/C#

**Formatting:**
- **CompressingTurtleWriter**: Full range of syntax compressions, configurable compression levels
- **TurtleFormatter**: QName compression
- **TurtleW3CFormatter**: W3C syntax with wider QName support
- Thread-safe writer implementation

---

### 16. ruby-rdf/rdf-turtle

**Ecosystem:** Ruby

**Formatting:** Streaming writer, configurable `base_uri` and `prefixes`, `standard_prefixes: true`
**Validation:** `validate` and `canonicalize` options, RDF 1.2 support

---

### 17. Oxigraph / oxttl

**Ecosystem:** Rust

**Features:**
- N-Triples, N-Quads, Turtle, TriG, N3 parsing and serialization
- Streaming parser (sync + async/Tokio)
- RDF-star support
- No dependencies outside Rust stdlib
- Replaces the older `rio_turtle` crate

---

### 18. Eclipse RDF4J

**Ecosystem:** Java (Eclipse Foundation)

**Validation:**
- Full SHACL validation engine via `ShaclSail`
- Transaction-aware validation (validates during `commit()`)
- Efficient incremental validation (analyzes only transaction changes)
- SHACL Extensions (RSX)

---

### 19. TopBraid / TopQuadrant SHACL

**Ecosystem:** Java (Apache Jena-based)

**Validation:**
- Reference SHACL implementation (from SHACL spec editors)
- Full SHACL constraint validation and rule inferencing
- CLI utilities: `shaclvalidate` and `shaclinfer`
- Only supports Turtle (`.ttl`) input

---

### 20. VS Code Turtle Formatter (bjdmeest)

**Ecosystem:** VS Code extension (uses graphy.js)

**Features:** Format-on-save for Turtle files, basic reformatting

---

## Cross-Tool Feature Matrix

### Linting Rules Across Tools

| Rule / Check | rdflint | RIOT | rapper | turtle-validator | Stardog LS |
|---|---|---|---|---|---|
| Syntax validation | Yes | Yes | Yes | Yes | Yes |
| Undefined prefix | Yes | Yes | Yes | Yes | Yes |
| Unused prefix detection | No | No | No | No | No |
| IRI validation (bad chars) | Via Jena | Yes | Yes | No | No |
| Literal datatype checking | Yes | Yes | No | Yes (XSD) | No |
| Language tag validation | Via Jena | Yes | No | No | No |
| Duplicate triple detection | No | No | No | No | No |
| Undefined subject reference | Yes | No | No | No | No |
| Custom rules (SPARQL) | Yes | No | No | No | No |
| SHACL validation | Yes | No | No | No | Via ICV |
| Outlier detection | Yes | No | No | No | No |
| Encoding validation | Yes | No | No | No | No |
| Literal trim validation | Yes | No | No | No | No |

### Formatting Options Across Tools

| Feature | turtle-formatter | turtlefmt | prttl | RIOT | rdflib | otsrdflib |
|---|---|---|---|---|---|---|
| Indent style (space/tab) | Yes | No | Yes | Yes | No | No |
| Indent size | Yes | No | Yes | Yes | No | No |
| `a` for `rdf:type` | Yes | Yes | Yes | Yes | Yes | Yes |
| Prefix ordering | Yes | No | No | No | No | No |
| Subject ordering | Yes | No | Yes | Yes | No | Yes |
| Predicate ordering | Yes | No | Yes (presets) | Yes | Yes | No |
| Object ordering | Yes | No | No | No | No | No |
| Align predicates | Yes | No | No | Yes | No | No |
| Align objects | Yes | No | No | Yes | No | No |
| Blank node compression | Configurable | No | Yes | Yes | Yes | No |
| Quote style | 3 options | Double only | Double only | No | No | No |
| End-of-line style | LF/CR/CRLF | No | No | No | No | No |
| Git-diff optimized | No | No | Yes | No | Yes | No |
| Remove comments | No | No | Yes | No | No | No |
| Escape normalization | No | Yes | Yes | No | No | No |
| Literal short notation | No | Yes | Yes | No | No | No |
| Unused prefix removal | Yes | No | No | No | No | No |

---

## Key Takeaways

1. **No tool combines parsing + formatting + linting** the way Biome does for JS/CSS. A Biome implementation would be the first integrated Turtle toolchain.

2. **Duplicate triple detection is absent** from all tools surveyed. This is a gap Biome can fill.

3. **Unused prefix detection** is only handled by turtle-formatter (via `keepUnusedPrefixes: false`). No tool flags it as a lint diagnostic.

4. **turtle-formatter (atextor)** is the gold standard for configurability with 25+ options.

5. **prttl and rdflib's LongTurtle** are the only diff-optimized formatters, a feature increasingly valued in ontology development.

6. **IRI validation** is most thorough in Jena RIOT's `CheckerIRI` with illegal character detection, scheme-specific violations, and relative IRI resolution.

7. **SHACL validation** is a separate concern (requires shape definitions) and is out of scope for initial Biome Turtle support, but could be a future extension.
