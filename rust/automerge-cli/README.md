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

File outputs are staged in the destination directory and atomically replaced only after success, so
readers never observe a partially written regular file. Existing permissions are preserved; symlinks
and files with multiple hard links are rejected. This protects visibility and failure safety but does
not promise power-loss durability.

## Inspect and transfer changes

`heads` prints a document's current change heads, and `diff` prints the changes present in the second
document but not the first:

```sh
automerge-cli heads document.automerge
automerge-cli diff base.automerge updated.automerge
```

`extract` writes the appendable changes after a base document's heads. `apply` applies those chunks
to the same base document and writes a complete document:

```sh
automerge-cli extract base.automerge updated.automerge --out update.changes
automerge-cli apply base.automerge update.changes --out result.automerge
```

The `diff`, `extract`, and `apply` commands require file paths for their inputs. `extract` and
`apply` write to stdout when `--out` is omitted.

`import` and `export` support JSON and TOML with `--format`. TOML date-time values are imported as strings;
JSON null values cannot be represented by TOML and are rejected on export.

`changes` prints change metadata as JSON. Use `--after HASH` for changes not reachable from a base
change, `--hash HASH` for one change, and `--limit N` to cap the result. `verify` validates a document;
`verify --json` emits a machine-readable success or failure report and exits nonzero for invalid input.
