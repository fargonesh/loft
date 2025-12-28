# loft
An interpreted language built on top of rust, providing an easy, safe way to build modern applications.

### Notice
This project is still under heavy development. Consider contributing :)

## Installation
```bash
curl -fsSL https://loft.fargone.sh/install.sh | sh
```

Alternatively, build from source:

```bash
# Clone the repository
git clone https://github.com/fargonesh/loft.git
cd loft

# Build the project
cargo build --release
```
## Examples

Check out the `examples/` directory for comprehensive examples:

- **`showcase.lf`** - Complete language feature showcase
- **`enum_patterns.lf`** - Enums and pattern matching
- **`error_handling.lf`** - Error propagation with `?`
- **`modules/`** - Module system examples

## Development

### Releasing

To create a new release, use the provided release script:

```bash
# Create a release candidate (creates a branch and PR)
./scripts/release.sh 0.1.0-rc-3

# Create a stable release (creates a tag and pushes immediately)
./scripts/release.sh 1.0.0 --tag
```

The script will:
1. Update versions in all `Cargo.toml` files.
2. Update `Cargo.lock`.
3. Create a branch/PR or a git tag.
4. If it's a stable release (no `-rc` or `-beta`), it will also update the `latest` tag.

## License

This project is licensed under the MIT license. See [LICENSE](./LICENSE.md) for details.

## Links

- **Documentation**: [Full language guide](https://loft.fargone.sh/docs/introduction.md)
- **Package Registry**: [loft.fargone.sh](https://loft.fargone.sh)
