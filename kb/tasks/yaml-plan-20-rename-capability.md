# Plan 20: YAML Rename Capability

## Goal

Implement LSP "rename symbol" for YAML anchors and aliases. Placing the cursor on an anchor (`&name`) or alias (`*name`) and renaming updates all matching references in the same document.

## Implementation

### Single function in `yaml.rs`

**File:** `crates/biome_service/src/file_handlers/yaml.rs`

Add a `rename` function following the JS pattern:

```rust
fn rename(
    _path: &BiomePath,
    parse: AnyParse,
    symbol_at: TextSize,
    new_name: String,
) -> Result<RenameResult, WorkspaceError> {
    // 1. Find token at cursor position
    // 2. Determine if it's an anchor or alias token
    // 3. Extract the bare name (strip & or *)
    // 4. Find all anchors/aliases with the same name in the same document
    // 5. Build text edits replacing just the name portion
    // 6. Return RenameResult { range, indels }
}
```

### Logic

1. **Find token at cursor**: Walk `syntax.descendants_tokens(Direction::Next)` to find the token whose range contains `symbol_at`.

2. **Identify anchor or alias**: Check if the token kind is `ANCHOR_PROPERTY_LITERAL` or `ALIAS_LITERAL`.

3. **Extract name**: Strip the `&` or `*` prefix.

4. **Find document scope**: Walk ancestors to find the containing `YamlDocument` node. All matching anchors/aliases must be within the same document.

5. **Collect all matching tokens**: Walk descendants of the document, filter for `ANCHOR_PROPERTY_LITERAL` and `ALIAS_LITERAL` tokens, match by name.

6. **Build edits**: For each matching token, compute the range of just the name portion (after `&` or `*`) and create a text edit replacing it with `new_name`.

7. **Validation**: Check `new_name` is a valid YAML anchor name (no spaces, no special characters).

### Wire up capability

Change in `yaml.rs` capabilities:
```rust
analyzer: AnalyzerCapabilities {
    // ...
    rename: Some(rename),  // was None
    // ...
},
```

### No semantic model dependency (initially)

The rename function can work by directly traversing the document's syntax tree — the same approach the lint rules use. This keeps it simple and avoids coupling to the semantic model crate. If the semantic model is available, it can be used for O(1) lookups instead.

## Files to Modify

| File | Change |
|------|--------|
| `crates/biome_service/src/file_handlers/yaml.rs` | Add `rename` fn, wire to capabilities |

## Verification

1. `cargo check -p biome_service` compiles
2. Rename `&foo` → `&bar` updates all `*foo` → `*bar` in same document
3. Rename across `---` document boundaries does NOT cross-pollinate
4. Invalid names rejected with appropriate error
