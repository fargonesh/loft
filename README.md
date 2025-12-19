# loft

A modern, interpreted programming language with a focus on simplicity, safety, and developer experience.

🚀 **Phase 1 Complete!** - Production-ready with closures, enums, pattern matching, modules, and error handling.

## ✨ Key Features

### Language Features
- **Closures & Lambdas** - First-class functions with environment capture
- **Enums & Pattern Matching** - Type-safe error handling and state machines
- **Module System** - File-based imports with `learn` keyword
- **Error Propagation** - Elegant `?` operator for Result/Option types
- **Type Inference** - Optional type annotations with powerful inference
- **Async/Await** - Built-in support for asynchronous programming
- **Traits** - Flexible polymorphism through trait system

### Developer Experience
- **LSP Support** - Full IDE integration with VSCode
- **Clear Error Messages** - Helpful diagnostics with type information
- **Rich Standard Library** - Comprehensive builtins for common tasks
- **Package Registry** - Built-in dependency management at [loft.fargone.sh](https://loft.fargone.sh)

## 🚀 Quick Start

### Installation

The easiest way to install loft is using our installation script:

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

## 📖 Language Guide

### Hello World

```loft
term.println("Hello, loft!");
```

### Variables and Types

```loft
let x = 42;              // Immutable by default
let mut y = 10;          // Mutable variable
const PI = 3.14159;      // Constant

let name: str = "Alice"; // Type annotation
let numbers = [1, 2, 3]; // Array
```

### Functions

```loft
fn greet(name: str) -> str {
    return "Hello, " + name;
}

term.println(greet("World"));
```

### Closures

```loft
let multiplier = 3;
let triple = (x: num) => x * multiplier;

term.println(triple(5));  // 15
```

### Enums and Pattern Matching

```loft
enum Result {
    Ok(num),
    Err(str),
}

fn divide(a: num, b: num) -> Result {
    if b == 0 {
        return Result.Err("Division by zero");
    }
    return Result.Ok(a / b);
}

let result = divide(10, 2);
let message = match result {
    Result.Ok(val) => "Success!",
    Result.Err(err) => err,
};
```

### Error Propagation

```loft
enum Result {
    Ok(num),
    Err(str),
}

fn calculate() -> Result {
    let x = divide(10, 2)?;  // Unwrap or early return
    let y = divide(x, 0)?;    // This will return Err
    return Result.Ok(y);
}
```

### Modules

```loft
// math_utils.lf
teach fn square(x: num) -> num {
    return x * x;
}

// main.lf
learn "./math_utils";
let result = math_utils.square(5);
```

### Structs

```loft
def Point {
    x: num,
    y: num,
}

let p = Point { x: 10, y: 20 };
term.println(p.x);
```

### Traits and Implementations

```loft
trait Add {
    fn add(self: Self, other: Self) -> Self;
}

impl Add for num {
    fn add(self: num, other: num) -> num {
        return self + other;
    }
}
```

## 📚 Examples

Check out the `examples/` directory for comprehensive examples:

- **`showcase.lf`** - Complete language feature showcase
- **`enum_patterns.lf`** - Enums and pattern matching
- **`error_handling.lf`** - Error propagation with `?`
- **`modules/`** - Module system examples

## 🛠️ Development

### Running Tests

```bash
cargo test
```

### Building Documentation

```bash
mdbook build book/
```

### LSP Development

The language server is located in `src/lsp_main.rs` and provides:
- Hover information
- Go to definition
- Diagnostics
- Code completion

## 📦 Package Management

### Publishing Packages

1. Login via the CLI:
   ```bash
   loft login
   ```

2. Publish your package:
   ```bash
   loft publish
   ```

### Using Packages

Add dependencies to your `manifest.json`:

```json
{
  "name": "my-project",
  "version": "0.1.0",
  "dependencies": {
    "http": "^1.0.0"
  }
}
```

## 🎯 Use Cases

loft is production-ready for:

- ✅ Command-line tools
- ✅ Scripting and automation
- ✅ Small to medium applications
- ✅ Systems with robust error handling
- ✅ Projects requiring code organization
- ✅ Functional programming patterns

## 🤝 Contributing

We welcome contributions! See the [adoption timeline](ADOPTION_TIMELINE.md) for current priorities.

## 📄 License

This project is open source. See LICENSE for details.

## 🔗 Links

- **Documentation**: [Full language guide](./book/)
- **Implementation Report**: [See what's complete](IMPLEMENTATION_REPORT.md)
- **Adoption Timeline**: [Development roadmap](ADOPTION_TIMELINE.md)
- **Package Registry**: [loft.fargone.sh](https://loft.fargone.sh)

### Hello World

Create a file `hello.lf`:

```loft
term.println("Hello, World!");
```

Run it:

```bash
loft hello.lf
```

### REPL

Start the interactive REPL:

```bash
loft
```

## Documentation

Comprehensive documentation is available in the [book](./book/src/) directory.

Key topics:
- [Getting Started](./book/src/getting-started.md)
- [Language Basics](./book/src/basics.md)
- [Builtin Functions](./book/src/builtins.md)
- [Package Manager](./book/src/package-manager.md)
- [LSP Integration](./book/src/lsp.md)

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Running Examples

```bash
loft examples/trait_example.lf
```

## IDE Support

### VSCode

An official VSCode extension is available in the `.vscode-extension` directory.

Features:
- Syntax highlighting
- Code completion
- Hover information
- Error diagnostics
- Go to definition

Install:
```bash
cd .vscode-extension
npm install
npm run install-ext
```

## Project Structure

- `src/` - Main interpreter source code
  - `parser/` - Parser implementation
  - `runtime/` - Runtime and interpreter
  - `builtin/` - Builtin functions
  - `lsp/` - Language Server Protocol
- `book/` - Documentation source
- `.vscode-extension/` - VSCode extension
- `registry/` - Package registry server
- `examples/` - Example loft programs

## License

See LICENSE file for details.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](./book/src/contributing.md) for guidelines.
