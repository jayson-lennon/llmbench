---
expected = "13"
---

What does the following Rust program print?

```rust
fn f(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => f(n - 1) + f(n - 2),
    }
}

fn main() {
    print!("{}", f(7));
}
```

# **OUTPUT FORMAT**

A single line with the number. No additional commentary.
Example: `8`
