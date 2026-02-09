# Fix P1B: Add Anchor/Tag/Alias Support to YAML Parser

## Status: COMPLETE

## Context

The YAML parser does not recognize `&anchor`, `!tag`, or `*alias` syntax. These characters (`&`, `!`, `*`) fall through to `consume_unexpected_token()` in the lexer, creating `ERROR_TOKEN` nodes. The AST grammar and generated syntax nodes already have full support for properties and aliases (slots, accessor methods, node kinds), but the lexer never emits the tokens and the parser never creates the nodes.

This causes the P1B formatter bug: `defaults: &defaults\n  timeout: 30` produces broken output because the parser doesn't build the correct AST.

## Step 1: Add Lexer Token Emission

**File**: `crates/biome_yaml_parser/src/lexer/mod.rs`

### 1a. New functions

**`consume_anchor_property()`** — `&name` → `ANCHOR_PROPERTY_LITERAL`
```rust
fn consume_anchor_property(&mut self) -> LexToken {
    self.assert_byte(b'&');
    let start = self.current_coordinate;
    self.advance(1); // skip '&'
    // Consume anchor name: ns-anchor-char = ns-char - c-flow-indicator
    while let Some(c) = self.current_byte() {
        if is_blank(c) || is_flow_collection_indicator(c) {
            break;
        }
        self.advance(1);
    }
    LexToken::new(ANCHOR_PROPERTY_LITERAL, start, self.current_coordinate)
}
```

**`consume_tag_property()`** — `!tag`/`!!type`/`!<uri>` → `TAG_PROPERTY_LITERAL`
```rust
fn consume_tag_property(&mut self) -> LexToken {
    self.assert_byte(b'!');
    let start = self.current_coordinate;
    self.advance(1); // skip '!'
    // Handle variants: !!type, !<uri>, !prefix!suffix, !suffix, bare !
    match self.current_byte() {
        Some(b'<') => {
            // Verbatim tag: !<uri>
            self.advance(1);
            while let Some(c) = self.current_byte() {
                self.advance(1);
                if c == b'>' { break; }
            }
        }
        Some(b'!') => {
            // Secondary tag handle: !!type
            self.advance(1);
            while let Some(c) = self.current_byte() {
                if is_blank(c) || is_flow_collection_indicator(c) { break; }
                self.advance(1);
            }
        }
        Some(c) if !is_blank(c) && !is_flow_collection_indicator(c) => {
            // Primary tag or named handle: !suffix or !prefix!suffix
            while let Some(c) = self.current_byte() {
                if is_blank(c) || is_flow_collection_indicator(c) { break; }
                self.advance(1);
            }
        }
        _ => {} // Bare ! (non-specific tag)
    }
    LexToken::new(TAG_PROPERTY_LITERAL, start, self.current_coordinate)
}
```

**`consume_alias()`** — `*name` → `ALIAS_LITERAL`
```rust
fn consume_alias(&mut self) -> LexToken {
    self.assert_byte(b'*');
    let start = self.current_coordinate;
    self.advance(1); // skip '*'
    while let Some(c) = self.current_byte() {
        if is_blank(c) || is_flow_collection_indicator(c) { break; }
        self.advance(1);
    }
    LexToken::new(ALIAS_LITERAL, start, self.current_coordinate)
}
```

### 1b. Modify `consume_tokens()` dispatch (line ~61)

Add three cases before the `_ => self.consume_unexpected_token()` fallthrough:

```rust
b'&' => self.consume_anchor_property().into(),
b'!' => self.consume_tag_property().into(),
b'*' => {
    // Aliases are flow nodes — wrap in FLOW_START/FLOW_END for block context
    let start = self.current_coordinate;
    let alias = self.consume_alias();
    let mut tokens = LinkedList::new();
    tokens.push_back(LexToken::pseudo(FLOW_START, start));
    tokens.push_back(alias);
    let mut trivia = self.consume_trivia(true);
    tokens.append(&mut trivia);
    tokens.push_back(LexToken::pseudo(FLOW_END, self.current_coordinate));
    tokens
},
```

### 1c. Modify `consume_flow_collection()` dispatch (line ~343)

Add handling inside the match in the while loop, before `_ => self.consume_unexpected_token()`:

```rust
(b'&', _) => self.consume_anchor_property(),
(b'!', _) => self.consume_tag_property(),
(b'*', _) => self.consume_alias(),
```

## Step 2: Add Parser Property Parsing

**File**: `crates/biome_yaml_parser/src/parser/block.rs`

### 2a. New `parse_properties()` function

```rust
fn parse_properties(p: &mut YamlParser) {
    if p.at(ANCHOR_PROPERTY_LITERAL) {
        let m = p.start();
        let anchor_m = p.start();
        p.bump(ANCHOR_PROPERTY_LITERAL);
        anchor_m.complete(p, YAML_ANCHOR_PROPERTY);
        if p.at(TAG_PROPERTY_LITERAL) {
            let tag_m = p.start();
            p.bump(TAG_PROPERTY_LITERAL);
            tag_m.complete(p, YAML_TAG_PROPERTY);
        }
        m.complete(p, YAML_PROPERTIES_ANCHOR_FIRST);
    } else if p.at(TAG_PROPERTY_LITERAL) {
        let m = p.start();
        let tag_m = p.start();
        p.bump(TAG_PROPERTY_LITERAL);
        tag_m.complete(p, YAML_TAG_PROPERTY);
        if p.at(ANCHOR_PROPERTY_LITERAL) {
            let anchor_m = p.start();
            p.bump(ANCHOR_PROPERTY_LITERAL);
            anchor_m.complete(p, YAML_ANCHOR_PROPERTY);
        }
        m.complete(p, YAML_PROPERTIES_TAG_FIRST);
    }
}

fn is_at_properties(p: &YamlParser) -> bool {
    p.at(ANCHOR_PROPERTY_LITERAL) || p.at(TAG_PROPERTY_LITERAL)
}
```

### 2b. Modify `parse_any_block_node()` (line 25)

Add a branch for properties before the existing dispatch:

```rust
pub(crate) fn parse_any_block_node(p: &mut YamlParser) -> ParsedSyntax {
    if is_at_properties(p) {
        let m = p.start();
        parse_properties(p);

        if p.at(MAPPING_START) {
            p.bump(MAPPING_START);
            BlockMapEntryList.parse_list(p);
            p.expect(MAPPING_END);
            Present(m.complete(p, YAML_BLOCK_MAPPING))
        } else if p.at(SEQUENCE_START) {
            p.bump(SEQUENCE_START);
            BlockSequenceEntryList.parse_list(p);
            p.expect(SEQUENCE_END);
            Present(m.complete(p, YAML_BLOCK_SEQUENCE))
        } else if p.at(T![|]) {
            p.bump(T![|]);
            BlockHeaderList.parse_list(p);
            parse_block_content(p);
            Present(m.complete(p, YAML_LITERAL_SCALAR))
        } else if p.at(T![>]) {
            p.bump(T![>]);
            BlockHeaderList.parse_list(p);
            parse_block_content(p);
            Present(m.complete(p, YAML_FOLDED_SCALAR))
        } else {
            // Properties without a recognizable block node following
            Present(m.complete(p, YAML_BOGUS_BLOCK_NODE))
        }
    } else if p.at(MAPPING_START) {
        // ... existing code unchanged ...
```

### 2c. Modify `is_at_any_block_node()` (line 334)

Add property token checks:

```rust
pub(crate) fn is_at_any_block_node(p: &YamlParser) -> bool {
    p.at(MAPPING_START) || p.at(SEQUENCE_START) || p.at(FLOW_START)
        || p.at(T![|]) || p.at(T![>])
        || p.at(ANCHOR_PROPERTY_LITERAL) || p.at(TAG_PROPERTY_LITERAL)
}
```

## Step 3: Add Parser Alias Handling

**File**: `crates/biome_yaml_parser/src/parser/flow.rs`

### 3a. Modify `parse_any_flow_node()` (line 18)

Add alias handling:

```rust
pub(crate) fn parse_any_flow_node(p: &mut YamlParser) -> ParsedSyntax {
    if p.at(ALIAS_LITERAL) {
        let m = p.start();
        p.bump(ALIAS_LITERAL);
        Present(m.complete(p, YAML_ALIAS_NODE))
    } else if is_at_flow_json_node(p) {
        Present(parse_flow_json_node(p))
    } else if is_at_flow_yaml_node(p) {
        Present(parse_flow_yaml_node(p))
    } else {
        Absent
    }
}
```

### 3b. Add property parsing to flow nodes

In `parse_flow_yaml_node()`:
```rust
pub(crate) fn parse_flow_yaml_node(p: &mut YamlParser) -> CompletedMarker {
    let m = p.start();
    parse_properties(p);
    if p.at(PLAIN_LITERAL) {
        parse_plain_scalar(p);
    }
    m.complete(p, YAML_FLOW_YAML_NODE)
}
```

In `parse_flow_json_node()`:
```rust
pub(crate) fn parse_flow_json_node(p: &mut YamlParser) -> CompletedMarker {
    let m = p.start();
    parse_properties(p);
    // ... existing content parsing unchanged
    m.complete(p, YAML_FLOW_JSON_NODE)
}
```

### 3c. Update `is_at` checks for flow node recovery

In `FlowSequenceEntryRecovery::is_at_recovered()`, add `ALIAS_LITERAL` check.
In `FlowMapEntryRecovery::is_at_recovered()`, add `ALIAS_LITERAL` check.

## Step 4: Move `parse_properties` to shared location

Since properties are used in both block.rs and flow.rs, either:
- Make `parse_properties()` and `is_at_properties()` `pub(crate)` in block.rs and import in flow.rs
- Or create a new `properties.rs` module

Simplest: make them `pub(crate)` in block.rs since they're small functions.

## Step 5: Add formatter support for alias nodes

**File**: `crates/biome_yaml_formatter/src/yaml/auxiliary/alias_node.rs` (likely exists as generated stub)

Check if the generated formatter for `YamlAliasNode` needs any changes. It should just format the `ALIAS_LITERAL` token.

## Step 6: Update tests

1. Run parser tests — existing test files with anchors/aliases should now parse correctly
2. Accept parser snapshot updates with `cargo insta accept`
3. Run formatter tests — anchor_alias.yaml.snap should now produce correct output
4. Accept formatter snapshot updates

## Verification

1. `cargo build -p biome_yaml_parser` — compiles
2. `cargo test -p biome_yaml_parser` — parser tests pass (accept snapshots)
3. `cargo build -p biome_yaml_formatter` — compiles
4. Use `quick_test.rs` to verify:
   - `defaults: &defaults\n  timeout: 30` → produces correct AST with properties
   - `<<: *defaults` → produces correct AST with alias
5. `cargo test -p biome_yaml_formatter` — all tests pass (accept snapshots)

## Files Summary

| File | Action |
|------|--------|
| `crates/biome_yaml_parser/src/lexer/mod.rs` | Add `consume_anchor_property()`, `consume_tag_property()`, `consume_alias()` + dispatch |
| `crates/biome_yaml_parser/src/parser/block.rs` | Add `parse_properties()`, modify `parse_any_block_node()` and `is_at_any_block_node()` |
| `crates/biome_yaml_parser/src/parser/flow.rs` | Add alias handling in `parse_any_flow_node()`, properties in flow node parsers |
| Parser snapshot files | Accept updated snapshots |
| Formatter snapshot files | Accept updated snapshots |
