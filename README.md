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

## License

This project is licensed under the MIT license. See [LICENSE](./LICENSE.md) for details.

## Links

- **Documentation**: [Full language guide](./book/)
- **Implementation Report**: [See what's complete](IMPLEMENTATION_REPORT.md)
- **Adoption Timeline**: [Development roadmap](ADOPTION_TIMELINE.md)
- **Package Registry**: [loft.fargone.sh](https://loft.fargone.sh)
