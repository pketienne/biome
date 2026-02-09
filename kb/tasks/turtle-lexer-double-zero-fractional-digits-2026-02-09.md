# Issue: Lexer Doesn't Handle DOUBLE with Zero Fractional Digits

**Date:** 2026-02-09
**Status:** Open (low priority)
**Discovered during:** Literal short notation edge-case testing

## Problem

The W3C Turtle grammar defines DOUBLE as:

```
DOUBLE ::= [+-]? ([0-9]+ '.' [0-9]* EXPONENT | '.' [0-9]+ EXPONENT | [0-9]+ EXPONENT)
EXPONENT ::= [eE] [+-]? [0-9]+
```

The first alternative allows **zero fractional digits**: `1.E3` is valid (`[0-9]+` = `1`, `.`, `[0-9]*` = empty, `EXPONENT` = `E3`).

However, the lexer in `crates/biome_turtle_parser/src/lexer/mod.rs` (line 816-819) requires a digit after the dot before consuming it as part of a number:

```rust
Some(b'.') => {
    if self
        .byte_at(1)
        .is_some_and(|b| b.is_ascii_digit())  // <-- rejects 1.E3
    {
        self.advance(1); // skip .
        self.consume_digits();
        // ... check for exponent
    } else {
        // Dot is statement terminator
        TURTLE_INTEGER_LITERAL
    }
}
```

When the lexer encounters `1.E3`:
1. Consumes `1` as digits
2. Sees `.`, peeks at `E` — not a digit
3. Returns `1` as `TURTLE_INTEGER_LITERAL`, leaves `.` as statement terminator
4. `E3` becomes an unexpected name token — parse error

## Proposed Fix

Extend the lookahead at line 816-819 to also check for `e`/`E`:

```rust
Some(b'.') => {
    let next = self.byte_at(1);
    if next.is_some_and(|b| b.is_ascii_digit()) {
        // Existing path: 1.5, 1.5E3, etc.
        self.advance(1);
        self.consume_digits();
        if let Some(b'e' | b'E') = self.current_byte() {
            self.advance(1);
            if let Some(b'+' | b'-') = self.current_byte() {
                self.advance(1);
            }
            self.consume_digits();
            return TURTLE_DOUBLE_LITERAL;
        }
        TURTLE_DECIMAL_LITERAL
    } else if next.is_some_and(|b| b == b'e' || b == b'E') {
        // New path: 1.E3 (zero fractional digits)
        self.advance(1); // skip .
        self.advance(1); // skip e/E
        if let Some(b'+' | b'-') = self.current_byte() {
            self.advance(1);
        }
        self.consume_digits();
        TURTLE_DOUBLE_LITERAL
    } else {
        TURTLE_INTEGER_LITERAL
    }
}
```

## Risks

- Could affect parsing of edge cases like `1.` at end of statement (but the `else` branch still handles that)
- Need to verify no regressions with decimal literal parsing
- The new branch only triggers when the character after `.` is exactly `e`/`E`, so it shouldn't affect other cases

## Workaround

The formatter's `is_valid_double()` in `crates/biome_turtle_formatter/src/turtle/value/rdf_literal.rs` already rejects `1.E3` for short notation, ensuring formatted output is always round-trippable. If the lexer is fixed, the formatter restriction can be relaxed.

## Impact

Very low. `1.E3` is an extremely uncommon form in real-world Turtle data. The workaround in the formatter is sufficient for practical use.
