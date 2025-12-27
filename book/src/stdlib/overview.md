# Standard Library Overview

loft includes a comprehensive standard library with builtin modules.

## Available Modules

- **term**: Terminal I/O and logging
- **array**: Array manipulation
- **string**: String operations
- **math**: Mathematical functions
- **fs**: File system operations
- **time**: Time and date handling
- **json**: JSON parsing and serialization
- **web**: HTTP client

## Using Builtins

Builtin modules are available without imports:

```loft
term.println("Hello");
let len = array.len([1, 2, 3]);
let sqrt = math.sqrt(16);
```

## Documentation

Each builtin module is documented in the following sections.
See the generated stdlib documentation at /stdlib-docs/index.html.
