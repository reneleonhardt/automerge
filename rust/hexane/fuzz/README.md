# Hexane public API fuzzing

This standalone `cargo-fuzz` package exercises Hexane through its public API.
Each input drives bounded mutations, invariant checks, and save/load round trips
for column, delta-column, and prefix-column types.

Compile the target with:

```sh
cargo check --manifest-path rust/hexane/fuzz/Cargo.toml --bin public_api
```

From this directory, run it with `cargo-fuzz`:

```sh
cargo fuzz run public_api
```
