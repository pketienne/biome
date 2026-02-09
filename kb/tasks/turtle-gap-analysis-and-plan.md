# Turtle Gap Analysis: Current Biome State vs. External Tools

## Date: 2026-02-08

## Overview

Comparison of the current Biome Turtle implementation against features found in popular TTL tools, with a prioritized plan for closing the gaps.

---

## Current Biome Turtle Implementation State

### Parser (biome_turtle_parser) -- FUNCTIONAL
- Complete W3C Turtle syntax parsing
- Lexer handles all token types: IRIs, prefixed names, blank nodes, 4 string literal types, numeric/boolean literals, language tags, comments
- Error recovery with bogus nodes
- Snapshot test suite

### Formatter (biome_turtle_formatter) -- STUB ONLY
- Infrastructure in place (context, comments, trivia handling, generated dispatch)
- **All 23 node formatters use `format_verbatim_node`** -- zero actual formatting
- Only `FormatTurtleStatementList` has real logic (`f.join().entries(...)`)
- Configuration: indent style, indent width, line ending, line width

### Analyzer/Linter (biome_turtle_analyze) -- EMPTY
- Framework wired up (suppression support, code actions, fix-all pipeline)
- **Registry has zero rules registered**
- Smoke test only

### Configuration (biome_configuration/src/turtle.rs)
- Formatter: enabled, indent style, indent width, line ending, line width
- Linter: enabled flag only
- Assist: enabled flag only (default: off)

### Service Integration (biome_service) -- FUNCTIONAL
- File handler with parse, format, lint, code actions, fix-all
- Settings resolution with override support
- Debug capabilities (syntax tree, formatter IR)

---

## Gap Analysis

### Formatter Gaps (all nodes are verbatim)

Features from other tools that Biome is missing:

| Feature | Found In | Priority | Complexity |
|---------|----------|----------|------------|
| Basic indentation and line breaks | All tools | **P0** | Medium |
| Prefix declaration formatting | All tools | **P0** | Low |
| Triple formatting (subject + predicate-object) | All tools | **P0** | Medium |
| Predicate-object list with `;` separators | All tools | **P0** | Medium |
| Object list with `,` separators | All tools | **P0** | Low |
| Blank node property list `[ ... ]` | All tools | **P0** | Medium |
| Collection `( ... )` formatting | All tools | **P0** | Low |
| Literal formatting (string, numeric, boolean) | All tools | **P0** | Low |
| Blank lines between statement groups | turtle-formatter, Jena, rdflib | **P1** | Low |
| `rdf:type` -> `a` shorthand | turtlefmt, prttl, turtle-formatter | **P2** | Low |
| Quote normalization (`'` -> `"`) | turtlefmt, prttl | **P2** | Low |
| Escape normalization | turtlefmt, prttl | **P2** | Medium |
| Literal short notation (typed -> shorthand) | turtlefmt, prttl | **P2** | Medium |
| Prefix alignment (on `:`) | turtle-formatter | **P3** | Medium |
| Predicate alignment (vertical) | turtle-formatter, Jena | **P3** | High |
| Object alignment (vertical) | turtle-formatter | **P3** | High |
| First predicate on new line option | turtle-formatter | **P3** | Low |
| Prefix ordering | turtle-formatter, prttl | **P3** | Medium |
| Predicate ordering | turtle-formatter, prttl, rdflib | **P3** | High |
| Subject ordering | turtle-formatter, prttl, otsrdflib | **P3** | High |
| Directive style (`@prefix` vs `PREFIX`) | Jena | **P3** | Low |
| Diff-optimized output mode | prttl, rdflib LongTurtle | **P3** | Medium |
| Unused prefix removal | turtle-formatter | **P3** | Medium |
| Comma usage for repeated predicates | turtle-formatter | **P3** | Medium |

### Linter Gaps (no rules at all)

Rules found across tools that Biome should implement:

| Rule | Category | Found In | Priority | Complexity |
|------|----------|----------|----------|------------|
| `noUndefinedPrefix` | correctness | rdflint, Jena, rapper | **P0** | Medium |
| `noUnusedPrefix` | suspicious | turtle-formatter | **P0** | Medium |
| `noDuplicatePrefixDeclaration` | correctness | (common sense) | **P0** | Low |
| `noInvalidIri` | correctness | Jena CheckerIRI | **P1** | Medium |
| `noInvalidLanguageTag` | correctness | Jena CheckerLiteral | **P1** | Low |
| `noDuplicateTriple` | suspicious | (gap in all tools) | **P1** | High |
| `noLiteralTrimIssues` | suspicious | rdflint | **P2** | Low |
| `useShorthandRdfType` | style | turtlefmt, prttl, turtle-formatter | **P2** | Low |
| `useConsistentQuotes` | style | turtlefmt, prttl | **P2** | Low |
| `useConsistentDirectiveStyle` | style | Jena | **P2** | Low |
| `useSortedPrefixes` | style | turtle-formatter | **P3** | Medium |
| `useGroupedSubjects` | style | rdflib, turtle-formatter | **P3** | High |
| `usePrefixedNames` | style | (common practice) | **P3** | High |
| `noMalformedDatatype` | correctness | rdflint, Jena | **P3** | High |
| `noUndefinedSubjectReference` | correctness | rdflint | **P3** | High |

### Configuration Gaps

Turtle-specific formatter options missing from `TurtleFormatterConfiguration`:

| Option | Found In | Priority |
|--------|----------|----------|
| `quoteStyle` | turtle-formatter | **P2** |
| `useAForRdfType` | turtle-formatter, turtlefmt | **P2** |
| `alignPredicates` | turtle-formatter, Jena | **P3** |
| `firstPredicateInNewLine` | turtle-formatter | **P3** |
| `directiveStyle` | Jena | **P3** |
| `keepUnusedPrefixes` | turtle-formatter | **P3** |
| `prefixOrder` | turtle-formatter | **P3** |
| `predicateOrder` | turtle-formatter, prttl | **P3** |

---

## Implementation Plan

### Phase 1: Core Formatter (P0) -- Make formatting actually work

**Goal:** Replace all verbatim stubs with real formatting logic. This is the highest-impact work since the formatter currently does nothing.

**Tasks:**

1. **`FormatTurtleRoot`** -- Proper document structure
   - BOM handling
   - Delegate to statement list
   - Final newline

2. **`FormatTurtleStatementList`** -- Statement separation
   - Blank line between triples blocks
   - No blank line between consecutive prefix/base declarations
   - Blank line between directives section and first triple

3. **`FormatTurtlePrefixDeclaration`** -- `@prefix ns: <iri> .`
   - Single space between `@prefix`, namespace, IRI, and `.`
   - Preserve comment trivia

4. **`FormatTurtleBaseDeclaration`** -- `@base <iri> .`

5. **`FormatTurtleSparqlPrefixDeclaration`** -- `PREFIX ns: <iri>`

6. **`FormatTurtleSparqlBaseDeclaration`** -- `BASE <iri>`

7. **`FormatTurtleTriples`** -- Subject + predicate-object list
   - Subject on first line
   - Predicate-object list indented if multi-line
   - Trailing `.`

8. **`FormatTurtlePredicateObjectList`** -- Wrapper for pairs

9. **`FormatTurtlePredicateObjectPair`** -- verb + object list
   - Verb followed by space then objects

10. **`FormatTurtlePredicateObjectPairList`** -- `;`-separated pairs
    - Each pair on its own line, indented one level
    - `;` at end of each pair except last

11. **`FormatTurtleObjectList`** -- `,`-separated objects
    - Single line if fits; wrap with indent if not

12. **`FormatTurtleSubject`**, **`FormatTurtleVerb`**, **`FormatTurtleObject`** -- Dispatch wrappers

13. **`FormatTurtleIri`** -- IRI reference passthrough

14. **`FormatTurtlePrefixedName`** -- Prefixed name passthrough

15. **`FormatTurtleBlankNode`** -- Blank node label passthrough

16. **`FormatTurtleBlankNodePropertyList`** -- `[ ... ]`
    - Inline if single predicate-object pair
    - Expanded with indentation if multiple pairs

17. **`FormatTurtleCollection`** -- `( ... )`
    - Inline if short
    - Wrapped with indentation if long

18. **`FormatTurtleCollectionObjectList`** -- Objects within collection

19. **`FormatTurtleRdfLiteral`** -- String with optional lang tag / datatype

20. **`FormatTurtleString`** -- String literal passthrough

21. **`FormatTurtleNumericLiteral`** -- Number passthrough

22. **`FormatTurtleBooleanLiteral`** -- Boolean passthrough

23. **`FormatTurtleDatatypeAnnotation`** -- `^^<type>` annotation

**Expected output style:**
```turtle
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

<http://example.org/alice>
    a foaf:Person ;
    foaf:name "Alice" ;
    foaf:knows <http://example.org/bob>,
        <http://example.org/carol> .

<http://example.org/bob>
    a foaf:Person ;
    foaf:name "Bob" .
```

**Acceptance criteria:**
- Formatting is idempotent (format twice = same result)
- All trivia (whitespace, comments) is preserved
- Snapshot tests pass

---

### Phase 2: Essential Lint Rules (P0-P1) -- Make the linter useful

**Goal:** Implement the most valuable lint rules that catch real errors and common issues.

**Tasks:**

1. **`noUndefinedPrefix`** (correctness)
   - Walk all `@prefix`/`PREFIX` declarations to build prefix map
   - Check all `PNAME_LN`/`PNAME_NS` usages against the map
   - Error if prefix not declared
   - Requires file-level visitor

2. **`noUnusedPrefix`** (suspicious)
   - Build prefix map from declarations
   - Track all prefix usages in prefixed names
   - Warning for declared-but-never-used prefixes
   - Code action: remove unused prefix declaration

3. **`noDuplicatePrefixDeclaration`** (correctness)
   - Detect same prefix namespace declared twice
   - Error if IRIs differ; warning if identical (redundant)

4. **`noInvalidIri`** (correctness)
   - Check IRI tokens for disallowed characters (spaces, tabs, control chars, `<`, `>`)
   - Warn on scheme-specific issues
   - Based on Jena RIOT's CheckerIRI logic

5. **`noInvalidLanguageTag`** (correctness)
   - Validate language tags against BCP47 pattern: `[a-zA-Z]{1,8}(-[a-zA-Z0-9]{1,8})*`
   - Based on Jena RIOT's CheckerLiteral logic

**Acceptance criteria:**
- Each rule has positive and negative test cases
- Suppression comments work (`# biome-ignore`)
- Rules produce clear diagnostic messages with fix suggestions where applicable

---

### Phase 3: Style Rules & Configuration (P2) -- Polish and configurability

**Goal:** Add style enforcement rules and Turtle-specific configuration options.

**Tasks:**

1. Add `quoteStyle` to `TurtleFormatterConfiguration`
   - Options: `double` (default), `single`
   - Formatter normalizes string quotes accordingly

2. Add `useAForRdfType` to `TurtleFormatterConfiguration`
   - Default: `true`
   - Formatter writes `a` instead of `rdf:type` when enabled

3. Implement `useShorthandRdfType` lint rule (style)
   - Suggest `a` instead of `rdf:type` / `<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>`
   - Auto-fixable

4. Implement `useConsistentQuotes` lint rule (style)
   - Enforce `"` vs `'` consistency per configuration
   - Auto-fixable

5. Implement `useConsistentDirectiveStyle` lint rule (style)
   - Flag mixed `@prefix`/`PREFIX` or `@base`/`BASE` usage
   - Auto-fixable

6. Implement `noLiteralTrimIssues` lint rule (suspicious)
   - Detect leading/trailing whitespace in literal values
   - Warning with suggested fix

**Acceptance criteria:**
- Configuration options are documented and tested
- Style rules are auto-fixable
- `biome check --fix` applies fixes correctly

---

### Phase 4: Advanced Features (P3) -- Competitive differentiation

**Goal:** Features that would make Biome the best-in-class Turtle tool.

**Tasks (pick and prioritize):**

1. **`noDuplicateTriple`** lint rule
   - Detect identical subject-predicate-object patterns
   - Requires building a triple index (semantic-level analysis)
   - **No existing tool does this** -- unique to Biome

2. **Predicate alignment option** (`alignPredicates`)
   - Vertically align predicates within a subject block
   - Like Jena RIOT's `wide` indent style

3. **First predicate on new line option** (`firstPredicateInNewLine`)
   - Place first predicate on new line after subject

4. **Prefix ordering** (`prefixOrder`)
   - Sort prefix declarations (common prefixes first: rdf, rdfs, xsd, owl)

5. **Escape normalization**
   - Normalize string and IRI escapes to reduce escape count
   - Like turtlefmt and prttl

6. **Literal short notation**
   - Use short notation for booleans, integers, decimals, doubles when lexically equivalent
   - Like turtlefmt and prttl

7. **Assists:**
   - Sort prefix declarations
   - Remove unused prefixes
   - Convert full IRI to prefixed name (when prefix available)
   - Convert `rdf:type` to `a` shorthand

8. **`useGroupedSubjects`** lint rule
   - Detect triples with same subject appearing in separate blocks
   - Suggest merging with `;`

---

## Priority Summary

| Priority | Scope | Description |
|----------|-------|-------------|
| **P0** | Phase 1 + Phase 2 (rules 1-3) | Core formatter + essential correctness rules |
| **P1** | Phase 2 (rules 4-5) | IRI and language tag validation |
| **P2** | Phase 3 | Style rules + Turtle-specific config options |
| **P3** | Phase 4 | Advanced formatting, duplicate detection, assists |

---

## Unique Opportunities for Biome

Based on the research, these features would set Biome apart from all existing Turtle tools:

1. **Integrated linter + formatter** -- No existing tool combines both in a single pass
2. **`noDuplicateTriple`** -- Not implemented by any surveyed tool
3. **`noUnusedPrefix` as a lint diagnostic** -- Only turtle-formatter silently removes them; no tool reports them as diagnostics
4. **Auto-fixable style rules** -- No existing Turtle linter offers code actions
5. **LSP integration** -- Only Stardog has a Turtle language server; Biome would be the second with richer diagnostics
