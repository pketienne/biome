# Turtle Remaining Work

## Date: 2026-02-09

## Current State

- **Parser**: Complete W3C Turtle syntax parsing with error recovery
- **Formatter**: All 23 node formatters implemented with real formatting logic
- **Linter**: 10 nursery rules implemented (P0-P2 from gap analysis)
- **Snapshot Tests**: 13 formatter + 20 analyzer snapshot tests
- **Service Integration**: File handler, settings resolution, LSP capabilities wired up

---

## Remaining Work

### 1. Auto-Fixes for Existing Lint Rules

**Priority: High** -- Enables `biome check --fix` for Turtle files.

Several rules emit diagnostics but lack `BatchMutation`-based code actions:

| Rule | Fix Description |
|------|-----------------|
| `useShorthandRdfType` | Replace `rdf:type` with `a` |
| `useConsistentQuotes` | Replace `'string'` with `"string"` |
| `useConsistentDirectiveStyle` | Convert `PREFIX`/`BASE` to `@prefix`/`@base` (or vice versa) |
| `noUnusedPrefix` | Remove the unused `@prefix` declaration |
| `noDuplicatePrefixDeclaration` | Remove the duplicate declaration |
| `noLiteralTrimIssues` | Trim leading/trailing whitespace from literal value |

**Acceptance criteria:**
- Each auto-fix produces valid Turtle when applied
- `check_code_action` in analyzer snapshot tests validates fix correctness
- Snapshot tests capture the fix output in `.snap` files

---

### 2. Turtle-Specific Formatter Configuration Options

**Priority: High** -- Makes the formatter configurable beyond generic indent/width settings.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `quoteStyle` | `"double"` \| `"single"` | `"double"` | Preferred quote style for string literals |
| `useAForRdfType` | `bool` | `true` | Automatically write `a` instead of `rdf:type` |

**Tasks:**
- Add fields to `TurtleFormatterConfiguration` in `crates/biome_configuration/src/turtle.rs`
- Wire through `TurtleFormatterSettings` in `crates/biome_service/src/file_handlers/turtle.rs`
- Add fields to `TurtleFormatOptions` in `crates/biome_turtle_formatter/src/context.rs`
- Implement the logic in the relevant formatter nodes
- Add snapshot tests with `.options.json` overrides

---

### 3. Suppression Comment Tests

**Priority: Medium** -- Verifies `# biome-ignore` works correctly for Turtle rules.

**Tasks:**
- Create `crates/biome_turtle_analyze/tests/suppression/` directory
- Add test cases for suppressing each rule with `# biome-ignore lint/nursery/ruleName`
- Wire up `run_suppression_test` in `spec_tests.rs` (following CSS pattern)
- Verify suppressed diagnostics don't appear in output

---

### 4. Additional Lint Rules (P3)

**Priority: Medium-Low** -- Advanced rules from the gap analysis.

| Rule | Category | Complexity | Description |
|------|----------|------------|-------------|
| `useSortedPrefixes` | style | Medium | Enforce alphabetical or conventional ordering of `@prefix` declarations |
| `useGroupedSubjects` | style | High | Detect triples with same subject in separate blocks; suggest merging with `;` |
| `usePrefixedNames` | style | High | Suggest prefixed names over full IRIs when a matching prefix is declared |
| `noMalformedDatatype` | correctness | High | Validate literal values conform to declared XSD datatypes |
| `noUndefinedSubjectReference` | correctness | High | Flag subjects referenced as objects but never defined as subjects |

---

### 5. Advanced Formatter Options (P3)

**Priority: Low** -- Competitive feature parity with turtle-formatter and Jena RIOT.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `alignPredicates` | `bool` | `false` | Vertically align predicates within a subject block |
| `firstPredicateInNewLine` | `bool` | `true` | Place first predicate on new line after subject |
| `directiveStyle` | `"turtle"` \| `"sparql"` | `"turtle"` | Prefer `@prefix`/`@base` vs `PREFIX`/`BASE` |
| `prefixOrder` | `string[]` | `[]` | Custom ordering for prefix declarations |
| `predicateOrder` | `string[]` | `["rdf:type"]` | Custom ordering for predicates within a subject |

---

### 6. Escape and Literal Normalization

**Priority: Low** -- Polish features found in turtlefmt and prttl.

| Feature | Description |
|---------|-------------|
| Escape normalization | Minimize string/IRI escape sequences (e.g. `\u0041` -> `A`) |
| Literal short notation | Convert `"true"^^xsd:boolean` -> `true`, `"42"^^xsd:integer` -> `42` |
| Quote promotion/demotion | Use triple quotes only for multiline strings |

---

### 7. Assists

**Priority: Low** -- Code actions that aren't tied to diagnostics.

| Assist | Description |
|--------|-------------|
| Sort prefix declarations | Reorder `@prefix` lines alphabetically |
| Remove unused prefixes | Bulk remove all unused prefix declarations |
| Convert IRI to prefixed name | Replace `<http://xmlns.com/foaf/0.1/name>` with `foaf:name` |
| Convert `rdf:type` to `a` | Replace all `rdf:type` usages with shorthand |

---

### 8. Documentation

**Priority: Medium** -- Required for website generation.

- Ensure each lint rule has complete rustdoc with examples (used by `gen-analyzer` codegen)
- Verify rule metadata (`version`, `language`, `recommended`, `sources`) is correct
- Add Turtle formatter section to website docs (PR to biomejs/website)

---

## Suggested Order of Implementation

1. Auto-fixes for existing rules (highest user impact)
2. `quoteStyle` + `useAForRdfType` config options
3. Suppression comment tests
4. Rule documentation polish
5. `useSortedPrefixes` rule
6. Advanced formatter options (`alignPredicates`, `firstPredicateInNewLine`)
7. Remaining P3 lint rules
8. Escape/literal normalization
9. Assists
