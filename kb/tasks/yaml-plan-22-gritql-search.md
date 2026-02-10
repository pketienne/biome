# Plan 22: Implement GritQL Search Support for YAML

## Context

Biome's `biome search` command uses GritQL for structural pattern matching against ASTs. Currently only JavaScript and CSS support search. Adding YAML support enables queries like `biome search 'YamlBlockMapImplicitEntry' --include='*.yaml'`. This requires registering YAML as a Grit target language, implementing a Grit parser adapter, and wiring it through the CLI and service layers.

## Overview of Changes

### Non-YAML crates (small, mechanical additions)
1. **Grit grammar keywords** — add `"yaml"` to keyword list
2. **Codegen config** — enable grit support for YAML language
3. **Generated grit mappings** — add YAML arm to codegen
4. **Target language macro** — add YAML to `generate_target_language!` and `generate_target_node!`
5. **CLI search routing** — add YAML arm to `is_file_compatible_with_pattern()`

### YAML-specific (new files + service wiring)
6. **Grit YAML parser** — new `grit_yaml_parser.rs`
7. **YAML target language** — new `yaml_target_language.rs` + `yaml_target_language/` module
8. **Service handler** — enable `search: Some(search)` in yaml.rs

### Not needed (for initial implementation)
- **Metavariable support** — YAML parser has no `YAML_METAVARIABLE` syntax kind. Metavariables (`$var` placeholders in Grit patterns) require lexer + syntax changes. For an initial implementation, pattern matching works against native AST node names without metavariables — same as CSS which has metavariable support but `name_for_kind()` and `named_slots_for_kind()` return stubs. We can add metavariable support later.

---

## Step-by-Step Implementation

### Step 1: Add `"yaml"` to Grit grammar keywords

**File:** `xtask/codegen/src/grit_kinds_src.rs`

Add `"yaml"` to the `keywords` array (after `"html"`):
```
"html",
"yaml",    // <-- NEW
```

This causes codegen to produce `YAML_KW` in `crates/biome_grit_syntax/src/generated/kind.rs`.

### Step 2: Enable Grit support for YAML in codegen

**File:** `xtask/codegen/src/language_kind.rs:172-174`

Change `supports_grit()`:
```rust
pub fn supports_grit(&self) -> bool {
    matches!(self, Self::Css | Self::Js | Self::Yaml)
}
```

### Step 3: Add YAML arm to `generate_grit_mappings`

**File:** `xtask/codegen/src/generate_grit_mappings.rs:139-154`

Add YAML to `LanguageConfig::new()`:
```rust
LanguageKind::Yaml => Self {
    syntax_kind_type: "YamlSyntaxKind",
    syntax_module: "biome_yaml_syntax",
    legacy_patterns: &[],
},
```

### Step 4: Run codegen

```bash
just gen-ast yaml
```

This generates:
- `crates/biome_grit_patterns/src/grit_target_language/yaml_target_language/constants.rs`
- `crates/biome_grit_patterns/src/grit_target_language/yaml_target_language/generated_mappings.rs`

Also regenerate grit syntax to get `YAML_KW`:
```bash
just gen-ast grit
```

### Step 5: Add YAML deps to biome_grit_patterns

**File:** `crates/biome_grit_patterns/Cargo.toml`

Add under `[dependencies]`:
```toml
biome_yaml_parser   = { workspace = true }
biome_yaml_syntax   = { workspace = true }
```

### Step 6: Add YAML to `generate_target_node!` macro

**File:** `crates/biome_grit_patterns/src/grit_target_node.rs`

Add import:
```rust
use biome_yaml_syntax::{YamlLanguage, YamlSyntaxKind, YamlSyntaxNode, YamlSyntaxToken};
```

Update macro invocation:
```rust
generate_target_node! {
    [CssLanguage, CssSyntaxNode, CssSyntaxToken, CssSyntaxKind],
    [JsLanguage, JsSyntaxNode, JsSyntaxToken, JsSyntaxKind],
    [YamlLanguage, YamlSyntaxNode, YamlSyntaxToken, YamlSyntaxKind]
}
```

### Step 7: Create YAML target language module

**File:** `crates/biome_grit_patterns/src/grit_target_language/yaml_target_language.rs` (NEW)

Follow CSS pattern (simplest existing implementation):

```rust
mod constants;
pub mod generated_mappings;

use super::GritTargetLanguageImpl;
use crate::{
    grit_target_node::GritTargetSyntaxKind,
};
use biome_yaml_syntax::{YamlLanguage, YamlSyntaxKind};
use biome_rowan::{RawSyntaxKind, SyntaxKindSet};
use generated_mappings::kind_by_name;

const COMMENT_KINDS: SyntaxKindSet<YamlLanguage> =
    SyntaxKindSet::from_raw(RawSyntaxKind(YamlSyntaxKind::COMMENT as u16));

#[derive(Clone, Debug)]
pub struct YamlTargetLanguage;

impl GritTargetLanguageImpl for YamlTargetLanguage {
    type Kind = YamlSyntaxKind;

    fn kind_by_name(&self, node_name: &str) -> Option<YamlSyntaxKind> {
        kind_by_name(node_name)
    }

    fn name_for_kind(&self, _kind: GritTargetSyntaxKind) -> &'static str {
        "(unknown node)"  // No legacy TreeSitter patterns for YAML
    }

    fn named_slots_for_kind(&self, _kind: GritTargetSyntaxKind) -> &'static [(&'static str, u32)] {
        &[]  // No legacy TreeSitter slot mappings for YAML
    }

    fn snippet_context_strings(&self) -> &[(&'static str, &'static str)] {
        &[
            ("", ""),
            ("GRIT_KEY: ", ""),          // value context
            ("GRIT_KEY:\n  ", ""),       // block value context
        ]
    }

    fn is_comment_kind(kind: GritTargetSyntaxKind) -> bool {
        kind.as_yaml_kind()
            .is_some_and(|kind| COMMENT_KINDS.matches(kind))
    }

    fn metavariable_kind() -> Self::Kind {
        // YAML doesn't have metavariable support yet.
        // Use YAML_BOGUS as a placeholder — it will never match any real node.
        YamlSyntaxKind::YAML_BOGUS
    }
}
```

Note: Using `YAML_BOGUS` as metavariable placeholder. Pattern matching against native Biome AST node names (e.g., `YamlBlockMapping`) works without metavariables. Metavariable support can be added later by adding `YAML_METAVARIABLE` to the syntax kind enum and lexer.

### Step 8: Create Grit YAML parser

**File:** `crates/biome_grit_patterns/src/grit_yaml_parser.rs` (NEW)

Follow `grit_css_parser.rs` pattern exactly:

```rust
use crate::{
    grit_analysis_ext::GritAnalysisExt,
    grit_target_language::GritTargetParser,
    grit_tree::GritTargetTree,
};
use biome_yaml_parser::parse_yaml;
use biome_yaml_syntax::YamlLanguage;
use biome_parser::AnyParse;
use camino::Utf8Path;
use grit_util::{AnalysisLogs, FileOrigin, Parser, SnippetTree};
use std::path::Path;

pub struct GritYamlParser;

impl GritTargetParser for GritYamlParser {
    fn from_cached_parse_result(
        &self,
        parse: &AnyParse,
        path: Option<&Path>,
        logs: &mut AnalysisLogs,
    ) -> Option<GritTargetTree> {
        for diagnostic in parse.diagnostics() {
            logs.push(diagnostic.to_log(path));
        }
        Some(GritTargetTree::new(parse.syntax::<YamlLanguage>().into()))
    }

    fn parse_with_path(&self, source: &str, _path: &Utf8Path) -> AnyParse {
        parse_yaml(source).into()
    }
}

impl Parser for GritYamlParser {
    type Tree = GritTargetTree;

    fn parse_file(
        &mut self,
        body: &str,
        path: Option<&Path>,
        logs: &mut AnalysisLogs,
        _old_tree: FileOrigin<'_, GritTargetTree>,
    ) -> Option<GritTargetTree> {
        let parse_result = parse_yaml(body);
        for diagnostic in parse_result.diagnostics() {
            logs.push(diagnostic.to_log(path));
        }
        Some(GritTargetTree::new(parse_result.syntax().into()))
    }

    fn parse_snippet(
        &mut self,
        prefix: &'static str,
        source: &str,
        postfix: &'static str,
    ) -> SnippetTree<GritTargetTree> {
        let context = format!("{prefix}{source}{postfix}");
        let len = if cfg!(target_arch = "wasm32") {
            |src: &str| src.chars().count() as u32
        } else {
            |src: &str| src.len() as u32
        };
        let parse_result = parse_yaml(&context);
        SnippetTree {
            tree: GritTargetTree::new(parse_result.syntax().into()),
            source: source.to_owned(),
            prefix,
            postfix,
            snippet_start: (len(prefix) + len(source) - len(source.trim_start())),
            snippet_end: (len(prefix) + len(source.trim_end())),
        }
    }
}
```

Note: No `.allow_metavariables()` call since YAML parser doesn't support it yet.

### Step 9: Register YAML in target language dispatch

**File:** `crates/biome_grit_patterns/src/grit_target_language.rs`

Add module declaration and re-export:
```rust
mod yaml_target_language;
pub use yaml_target_language::YamlTargetLanguage;
```

Update macro invocation:
```rust
generate_target_language! {
    [CssTargetLanguage, GritCssParser, "CSS"],
    [JsTargetLanguage, GritJsParser, "JavaScript"],
    [YamlTargetLanguage, GritYamlParser, "YAML"]
}
```

Add import for the parser:
```rust
use crate::grit_yaml_parser::GritYamlParser;
```

Update `from_declaration()`:
```rust
GritSyntaxKind::YAML_KW => Some(Self::YamlTargetLanguage(YamlTargetLanguage)),
```

Update `from_extension()`:
```rust
"yaml" | "yml" => Some(Self::YamlTargetLanguage(YamlTargetLanguage)),
```

Also declare the parser module in the crate root:

**File:** `crates/biome_grit_patterns/src/lib.rs`

Add:
```rust
mod grit_yaml_parser;
```

### Step 10: Add CLI search file compatibility

**File:** `crates/biome_cli/src/execute/process_file/search.rs`

Update `is_file_compatible_with_pattern()`:
```rust
GritTargetLanguage::YamlTargetLanguage(_) => {
    matches!(file_source, DocumentFileSource::Yaml(_))
}
```

### Step 11: Enable search in YAML file handler

**File:** `crates/biome_service/src/file_handlers/yaml.rs`

Change `search: SearchCapabilities { search: None }` to:
```rust
search: SearchCapabilities { search: Some(search) },
```

And add a `search` function + `search_enabled` function following the CSS pattern.

---

## Files to modify (summary)

| File | Type | Change |
|------|------|--------|
| `xtask/codegen/src/grit_kinds_src.rs` | Modify | Add `"yaml"` keyword |
| `xtask/codegen/src/language_kind.rs` | Modify | Add `Self::Yaml` to `supports_grit()` |
| `xtask/codegen/src/generate_grit_mappings.rs` | Modify | Add YAML arm to `LanguageConfig::new()` |
| `crates/biome_grit_patterns/Cargo.toml` | Modify | Add yaml parser/syntax deps |
| `crates/biome_grit_patterns/src/lib.rs` | Modify | Add `mod grit_yaml_parser;` |
| `crates/biome_grit_patterns/src/grit_target_node.rs` | Modify | Add YAML to macro + imports |
| `crates/biome_grit_patterns/src/grit_target_language.rs` | Modify | Add YAML to macro, imports, `from_declaration`, `from_extension` |
| `crates/biome_grit_patterns/src/grit_yaml_parser.rs` | **New** | Grit parser adapter for YAML |
| `crates/biome_grit_patterns/src/grit_target_language/yaml_target_language.rs` | **New** | YAML target language impl |
| `crates/biome_grit_patterns/src/grit_target_language/yaml_target_language/constants.rs` | **Generated** | Disregarded snippet slots |
| `crates/biome_grit_patterns/src/grit_target_language/yaml_target_language/generated_mappings.rs` | **Generated** | Node name → syntax kind mappings |
| `crates/biome_grit_syntax/src/generated/kind.rs` | **Generated** | `YAML_KW` variant (after codegen) |
| `crates/biome_cli/src/execute/process_file/search.rs` | Modify | Add YAML file compatibility |
| `crates/biome_service/src/file_handlers/yaml.rs` | Modify | Enable search capability |

## Verification

1. `just gen-ast grit` — regenerates grit syntax with `YAML_KW`
2. `just gen-ast yaml` — generates `constants.rs` and `generated_mappings.rs`
3. `cargo build -p biome_grit_patterns` — compiles clean
4. `cargo test -p biome_grit_patterns` — all existing tests pass
5. `cargo build -p biome_cli` — CLI compiles with YAML search support
6. Manual test: `cargo biome-cli-dev search 'YamlBlockMapping' --include='*.yaml'` against a YAML file
