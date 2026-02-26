# loft Programming Language Guide

This directory contains the comprehensive documentation for the loft programming language in mdBook format.

## Building the Book

### Prerequisites

Install mdbook:
```bash
cargo install mdbook
```

### Build

```bash
cd book
mdbook build
```

The built book will be in `book/book/` directory.

### Serve Locally

To view the book with live reload:
```bash
mdbook serve
```

Then open http://localhost:3000 in your browser.

## Contents

The book covers:

- **Language Guide**: Variables, functions, control flow, advanced features
- **Builtin System**: Terminal, math, time, file system, string/array methods
- **Module System**: Project structure, imports/exports, package manager
- **Architecture**: Parser, runtime, LSP server, builtin design
- **Contributing**: How to contribute to loft

## Structure

```
book/
├── book.toml           # Book configuration
├── .gitignore          # Ignore build artifacts
└── src/
    ├── SUMMARY.md      # Table of contents
    ├── introduction.md # Introduction chapter
    ├── *.md            # Various chapter files
    └── ...
```

## Contributing

To contribute to the documentation:

1. Edit markdown files in `src/`
2. Build and preview: `mdbook serve`
3. Commit changes
4. Submit a pull request

## License

Same as the main loft project.
