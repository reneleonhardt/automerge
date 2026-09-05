# Automerge CLI

## Anonymize a document

The `anonymize` command replaces document data while retaining its history and structural shape:

```sh
cargo run -p automerge-cli -- anonymize input.automerge --out output.automerge
```

Input and output can instead be piped through stdin and stdout. The result still reveals metadata
such as the change graph, object and value types, collection sizes, string lengths, and whitespace.
Review it before publishing.

## Copy a document without reserializing

The `copy` command validates a complete Automerge document and preserves its exact bytes:

```sh
automerge-cli copy input.automerge --out output.automerge
```

Input and output can instead be piped through stdin and stdout. Unlike `merge`, this command does
not compact or normalize the document.
