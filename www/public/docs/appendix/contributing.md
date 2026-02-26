# Contributing

Thanks for your interest in contributing to loft!

## Getting Started

The project uses [devenv](https://devenv.sh) to manage the development environment. With devenv installed, run:

```bash
devenv shell
```

This provides Rust (nightly), Node.js, and all other required tools.

## Building

```bash
cargo build
```

## Running Tests

```bash
cargo test
```

## Adding Example Files

Example programs live in the `examples/` directory. The integration tests in
`tests/examples.rs` are **generated automatically** — do not edit that file by
hand.

When you add, rename, or remove a `.lf` file in `examples/`, regenerate the
test file by running the script from the workspace root:

```bash
bash scripts/gen_example_tests.sh
```

Then commit both the new example and the updated `tests/examples.rs` together.

A CI check (`check-generated-tests`) will diff the script's output against the
committed file and fail the build if they are out of sync, so you will be
reminded if you forget.

### Special cases

| Situation | What to do |
|-----------|------------|
| Example needs network access or an external package | Add its filename to the `IGNORED_FILE` list in `gen_example_tests.sh` with an appropriate reason string. It will be emitted as an `#[ignore]`d test. |
| Example requires a native shared library (FFI) | Name the file with the `ffi_` prefix. The script will comment it out automatically. |
| Example uses relative `learn` paths (modules) | Place it under `examples/modules/` — the script generates a dedicated test that sets the working directory correctly. |

## Code Style

Run the formatter before submitting a PR:

```bash
cargo fmt
cargo clippy -- -D warnings
```
