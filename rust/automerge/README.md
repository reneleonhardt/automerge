# Automerge

Automerge is a library of data structures for building collaborative
[local-first](https://www.inkandswitch.com/local-first/) applications. This is
the Rust implementation. See [automerge.org](https://automerge.org/) 

## Rust persistence API

`Automerge` and `AutoCommit` provide the low-level document and transaction APIs used by the
language bindings. For a normal full save, `save()` is the simplest choice. Applications that
repeatedly publish an unchanged snapshot can use `save_cached()` to receive an immutable,
reference-counted `Arc<[u8]>` without copying the cached bytes; the snapshot remains valid after
later mutations. Use `save_to(&mut writer)` to write through a caller-owned writer.
Incremental persistence remains available through `save_after()` and
`AutoCommit::save_incremental()`. These methods return appendable change chunks rather than a full
document snapshot, and their `*_to(&mut writer)` variants write those chunks directly to a
caller-owned writer.
