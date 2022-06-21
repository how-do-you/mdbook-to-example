<div align="center">

# mdbook-to-example

Turns an `mdbook` book into a Rust example that can be used to create documentation with `cargo doc`
</div>

## Getting started

```rust
// build.rs
fn main() -> Result<(), std::io::Error> {
    let _ = mdbook_to_example::Builder::new()
        .set_name("package-book")
        .run();
    Ok(())
}
```
