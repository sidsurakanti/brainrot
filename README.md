```text
// brain.rot
a = 1 * 2 + 3;
```
```rust
[
    Token {
        kind: Identifier(
            "a",
        ),
        span: 0..1,
    },
    Token {
        kind: Assign,
        span: 2..3,
    },
    Token {
        kind: Number(
            1,
        ),
        span: 4..5,
    },
    Token {
        kind: Times,
        span: 6..7,
    },
    Token {
        kind: Number(
            2,
        ),
        span: 8..9,
    },
    Token {
        kind: Plus,
        span: 10..11,
    },
    Token {
        kind: Number(
            3,
        ),
        span: 12..13,
    },
    Token {
        kind: Semicolon,
        span: 13..14,
    },
    Token {
        kind: EOF,
        span: 15..15,
    },
]
```
