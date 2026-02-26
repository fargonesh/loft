# loft
A cozy, interpreted language built on Rust. We're trying to make building modern apps actually feel good — safe, fast, and easy to read.

### Quick Heads Up
This project is still a work in progress! We'd love for you to jump in and help out. :)

## Getting Started
To get loft up and running on your machine:
```bash
curl -fsSL https://loft.fargone.sh/install.sh | sh
```

If you prefer building from source (you're cool like that):

```bash
# Grab the code
git clone https://github.com/fargonesh/loft.git
cd loft

# Let's go!
cargo build --release
```

## Making it Better

We use a Nix flake for development. You can jump into a shell with everything pre-configured, or just spin up the dev servers.

### Spin up the Dev Servers
This starts the backend (package registry) and the frontend (web server) all at once:

```bash
nix run .#serve
```

### Jump into the Shell
Get a fresh environment with all the tools you need:

```bash
nix develop
```

If you're a `devenv` fan:

```bash
# Start everything up
devenv up
```

## Examples

Take a peek at the `examples/` folder to see what loft can do:

- **`showcase.lf`** - Complete language feature showcase
- **`enum_patterns.lf`** - Enums and pattern matching
- **`error_handling.lf`** - Error propagation with `?`
- **`modules/`** - Module system examples

## License

This project is licensed under the MIT license. See [LICENSE](./LICENSE.md) for details.

## Links

- **Documentation**: [Full language guide](https://loft.fargone.sh/docs/introduction.md)
- **Package Registry**: [loft.fargone.sh](https://loft.fargone.sh)
