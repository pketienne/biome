# Plan 21: Lexer `rewind()` Implementation

## Goal

Implement the `rewind()` method on `YamlLexer` so it satisfies the `Lexer` trait contract. Currently it panics with `unimplemented!()`. While no code path currently calls it, implementing it removes a latent panic risk and brings the YAML lexer in line with JS and CSS lexers.

## Implementation

### Checkpoint creation

The `Lexer` trait's `checkpoint()` method (default implementation in `biome_parser/src/lexer.rs`) creates a `LexerCheckpoint` with:
- `position: TextSize` — byte offset in source
- `current_start: TextSize` — start of current token
- `current_kind: Kind` — current token kind
- `current_flags: TokenFlags` — token flags
- `after_line_break: bool`
- `unicode_bom_length: usize`
- `diagnostics_pos: u32` — number of diagnostics at checkpoint time

The YAML lexer needs to store additional state beyond what `LexerCheckpoint` captures, because it has:
- `scopes: Vec<BlockScope>` — indentation scope stack
- `tokens: VecDeque<LexToken>` — buffered token queue
- `current_coordinate: TextCoordinate` — offset + column position

### Approach: Store extra state in a side table

Since `LexerCheckpoint` is a generic struct we can't extend, we store the extra YAML-specific state in the lexer itself, keyed by the checkpoint position:

```rust
pub(crate) struct YamlLexer<'src> {
    source: &'src str,
    current_coordinate: TextCoordinate,
    diagnostics: Vec<ParseDiagnostic>,
    scopes: Vec<BlockScope>,
    tokens: VecDeque<LexToken>,
    // NEW: saved checkpoints for rewind support
    saved_states: Vec<SavedLexerState>,
}

#[derive(Clone)]
struct SavedLexerState {
    position: TextSize,
    coordinate: TextCoordinate,
    scopes: Vec<BlockScope>,
    tokens: VecDeque<LexToken>,
    diagnostics_len: usize,
}
```

### `checkpoint()` override

Override the default `checkpoint()` to also save YAML-specific state:

```rust
fn checkpoint(&self) -> LexerCheckpoint<Self::Kind> {
    let cp = LexerCheckpoint {
        position: TextSize::from(self.current_coordinate.offset as u32),
        current_start: self.current_start(),
        current_kind: self.current(),
        current_flags: TokenFlags::empty(),
        after_line_break: false,
        unicode_bom_length: 0,
        diagnostics_pos: self.diagnostics.len() as u32,
    };
    self.saved_states.push(SavedLexerState {
        position: cp.position,
        coordinate: self.current_coordinate,
        scopes: self.scopes.clone(),
        tokens: self.tokens.clone(),
        diagnostics_len: self.diagnostics.len(),
    });
    cp
}
```

### `rewind()` implementation

```rust
fn rewind(&mut self, checkpoint: LexerCheckpoint<Self::Kind>) {
    // Find and remove the saved state matching this checkpoint
    if let Some(idx) = self.saved_states.iter().position(|s| s.position == checkpoint.position) {
        let saved = self.saved_states.remove(idx);
        self.current_coordinate = saved.coordinate;
        self.scopes = saved.scopes;
        self.tokens = saved.tokens;
        self.diagnostics.truncate(saved.diagnostics_len);
    }
    // Clear any saved states created after this checkpoint
    self.saved_states.retain(|s| s.position <= checkpoint.position);
}
```

### Derive Clone for BlockScope

`BlockScope` must derive `Clone` for the saved state. Check if it already does; if not, add `#[derive(Clone)]`.

### Derive Clone for LexToken

Similarly, `LexToken` must derive `Clone` (or `Copy` if it already does — it's likely `Copy` since it's small).

## Files to Modify

| File | Change |
|------|--------|
| `crates/biome_yaml_parser/src/lexer/mod.rs` | Add `saved_states` field, implement `checkpoint()` + `rewind()`, derive `Clone` on `BlockScope` |

## Verification

1. `cargo check -p biome_yaml_parser` compiles
2. All existing parser tests still pass
3. Add a unit test that creates a checkpoint, advances, rewinds, and verifies state is restored
4. `cargo clippy -p biome_yaml_parser` — zero warnings
