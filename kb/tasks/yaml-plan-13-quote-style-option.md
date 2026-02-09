# Plan 13: Quote Style Formatter Option

## Status: PENDING

## Context

The YAML formatter currently preserves whatever quoting style the input uses. Users should be able to configure a preferred quote style (single or double) to enforce consistency, similar to how the JS formatter handles `quote_style`.

The `QuoteStyle` enum already exists in `biome_formatter/src/lib.rs` with `Double` (default) and `Single` variants, plus helper methods (`as_char()`, `as_byte()`, `other()`).

## Changes

### 13A. Add quote_style to YamlFormatOptions

**File**: `crates/biome_yaml_formatter/src/context.rs`

```rust
use biome_formatter::QuoteStyle;

#[derive(Debug, Default, Clone)]
pub struct YamlFormatOptions {
    indent_style: IndentStyle,
    indent_width: IndentWidth,
    line_ending: LineEnding,
    line_width: LineWidth,
    quote_style: QuoteStyle,  // NEW
}

// Add builder method:
pub fn with_quote_style(mut self, quote_style: QuoteStyle) -> Self {
    self.quote_style = quote_style;
    self
}

// Add setter:
pub fn set_quote_style(&mut self, quote_style: QuoteStyle) {
    self.quote_style = quote_style;
}

// Add getter:
pub fn quote_style(&self) -> QuoteStyle {
    self.quote_style
}
```

### 13B. Add quote_style to YamlFormatterConfiguration

**File**: `crates/biome_configuration/src/yaml.rs`

```rust
use biome_formatter::QuoteStyle;

pub struct YamlFormatterConfiguration {
    // ... existing fields ...

    /// The type of quotes used in YAML strings. Defaults to double.
    #[bpaf(long("yaml-formatter-quote-style"), argument("double|single"), optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_style: Option<QuoteStyle>,
}
```

### 13C. Wire through service layer

**File**: `crates/biome_service/src/file_handlers/yaml.rs`

Add `quote_style` field to `YamlFormatterSettings`:
```rust
pub struct YamlFormatterSettings {
    // ... existing fields ...
    pub quote_style: Option<QuoteStyle>,
}
```

Update `resolve_format_options()`:
```rust
let quote_style = language.quote_style.unwrap_or_default();

let mut options = YamlFormatOptions::default()
    // ... existing options ...
    .with_quote_style(quote_style);
```

### 13D. Update override settings

**File**: `crates/biome_service/src/settings.rs`

In `apply_overrides_to_yaml_format_options()`, add:
```rust
if let Some(quote_style) = &self.formatter.quote_style {
    options.set_quote_style(*quote_style);
}
```

### 13E. Implement quote normalization in scalar formatters

**File**: `crates/biome_yaml_formatter/src/yaml/auxiliary/double_quoted_scalar.rs`

```rust
impl FormatNodeRule<YamlDoubleQuotedScalar> for FormatYamlDoubleQuotedScalar {
    fn fmt_fields(
        &self,
        node: &YamlDoubleQuotedScalar,
        f: &mut YamlFormatter,
    ) -> FormatResult<()> {
        let quote_style = f.options().quote_style();
        let token = node.value_token()?;
        let text = token.text_trimmed();

        if quote_style.is_double() {
            // Already double-quoted, format as-is
            token.format().fmt(f)
        } else {
            // Convert to single quotes if possible (no single quotes in content)
            let content = &text[1..text.len() - 1]; // strip surrounding quotes
            if !content.contains('\'') {
                // Safe to convert: replace outer quotes
                write!(f, [format_replaced(
                    &token,
                    &dynamic_token(&format!("'{content}'"), token.text_trimmed_range().start())
                )])
            } else {
                // Content contains single quotes — keep double quotes to avoid escaping
                token.format().fmt(f)
            }
        }
    }
}
```

**File**: `crates/biome_yaml_formatter/src/yaml/auxiliary/single_quoted_scalar.rs`

Similar logic but reversed — convert to double quotes when `quote_style` is `Double`.

### 13F. Add formatter test

**File**: `crates/biome_yaml_formatter/tests/specs/yaml/scalar/quote_style.yaml`

Test file with mixed quoting styles to verify normalization.

## Verification
1. `cargo build -p biome_yaml_formatter` — compiles
2. `cargo test -p biome_yaml_formatter` — all tests pass
3. Test with `biome format --yaml-formatter-quote-style=single test.yaml`
4. Verify escaping edge cases (strings containing the target quote character)

## Notes
- YAML has three string styles: plain, single-quoted, double-quoted
- Plain scalars should NOT be quoted by the formatter (that would change semantics for some values)
- Double-quoted strings support escape sequences (`\n`, `\t`, etc.) — converting to single quotes is only safe when no escape sequences are present
- Single-quoted strings only support `''` (escaped single quote) — converting to double quotes requires escaping backslashes
