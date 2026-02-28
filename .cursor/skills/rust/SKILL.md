---
name: rust
description: Provides Rust coding best practices for structure, patterns, performance, and error handling. Use when writing or reviewing Rust code, or when the user asks for Rust style guidance.
---

# Rust Best Practices

## Quick Start

Apply these rules by default when touching Rust:

1. Organize code by feature/module, not by file type
2. Keep structs small and focused; split large data into composable types
3. Prefer `Result<T, E>` for recoverable errors, avoid `panic!`
4. Document every `unsafe` block with a `// SAFETY:` rationale
5. Default to `Vec`/`HashMap`; pre-allocate when size is known

## Workflow (use this order)

1. Clarify scope: new module, refactor, or review.
2. Organize by feature/module; keep type + impls together.
3. Ensure error handling uses `Result` (avoid `panic!` except invariants).
4. Document `unsafe` blocks with `// SAFETY:` and minimal scope.
5. Choose data structures intentionally and pre-allocate when size is known.
6. Add or update tests (unit tests + rustdoc examples for public APIs).

## Review Checklist

- Module layout is feature-driven and cohesive.
- `Result` used for recoverable errors; `panic!` justified.
- All `unsafe` blocks have explicit `// SAFETY:` rationale.
- No oversized structs; data is composed cleanly.
- Collections pre-allocated when size is known.
- Tests and doc examples cover edge cases.

## Local Resources

Use the always-applied rule file in this folder:
- `rust.mdc` (authoritative Rust guidance; read and apply the full rule text)

## Code Organization

- **Feature-driven modules**: Keep a struct, its enums, and `impl` blocks together
- **Small, cohesive types**: Split large structs into composable pieces

## Common Patterns

- **Newtype for type safety**: Wrap primitives to avoid ID/value mixups
- **Builder for complex construction**: Use builders for many optional fields
- **Minimal generic bounds**: Put bounds on `impl`/functions, not the type
  - Favor `struct Foo<T> { ... }` with bounds on `impl Foo<T>` or methods

## Performance Guidance

- Default to `Vec` and `HashMap`; switch only with evidence or requirements
- Pre-allocate capacity when the approximate size is known

## Error Handling and Safety

- Use `Result<T, E>` for recoverable errors
- Reserve `panic!` for invariants or unrecoverable bugs
- Add a `// SAFETY:` rationale before every `unsafe` block

## Testing

- Prefer unit tests with `#[test]` in a `tests` module
- Use `rustdoc` examples for public APIs to keep docs executable

## Examples

**Example 1: Newtype for IDs**

Input: "Function accepts multiple `u64` IDs"
Output:
```rust
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UserId(u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AccountId(u64);

fn process_transaction(user_id: UserId, account_id: AccountId) { /* ... */ }
```

**Example 2: Documenting `unsafe`**

Input: "Need to write through a raw pointer"
Output:
```rust
let mut vec = vec![0];
let ptr = vec.as_mut_ptr();
// SAFETY: `ptr` is valid for `vec[0]` and we write a valid value.
unsafe { *ptr = 42; }
```

**Example 3: Builder for optional config**

Input: "Many optional config fields with defaults"
Output:
```rust
pub struct Config { pub timeout: u64, pub retries: u8 }

#[derive(Default)]
pub struct ConfigBuilder { timeout: Option<u64>, retries: Option<u8> }

impl Config {
    pub fn builder() -> ConfigBuilder { ConfigBuilder::default() }
}

impl ConfigBuilder {
    pub fn timeout(mut self, timeout: u64) -> Self { self.timeout = Some(timeout); self }
    pub fn retries(mut self, retries: u8) -> Self { self.retries = Some(retries); self }
    pub fn build(self) -> Config {
        Config {
            timeout: self.timeout.unwrap_or(1000),
            retries: self.retries.unwrap_or(3),
        }
    }
}
```

**Example 4: Pre-allocate collection capacity**

Input: "Known item count during collection building"
Output:
```rust
let mut items = Vec::with_capacity(1000);
for i in 0..1000 {
    items.push(i);
}
```
