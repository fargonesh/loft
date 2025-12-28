# Modules and Imports

loft's module system uses `learn` for imports and `teach` for exports.

## Importing Modules

Use `learn` to import:

```loft
learn "math";

let result = math.sqrt(16);
term.println(result);  // 4
```

Import with specific path:

```loft
learn "std/fs";

let content = fs.read("file.txt");
```

## Exporting Symbols

Use `teach` to export functions and values:

```loft
// In math_utils.lf
teach fn add(a: num, b: num) -> num {
    return a + b;
}

teach fn multiply(a: num, b: num) -> num {
    return a * b;
}

teach const PI = 3.14159;
```

Then import in another file:

```loft
learn "math_utils";

let sum = math_utils.add(5, 3);
let product = math_utils.multiply(4, 7);
term.println(math_utils.PI);
```

## Module Structure

Modules can be single files or directories:

```
project/
  main.lf
  utils.lf
  math/
    operations.lf
    constants.lf
```

Import from directories:

```loft
learn "math/operations";
```

## Builtin Modules

loft includes several builtin modules:

- `term`: Terminal I/O
- `array`: Array utilities
- `string`: String operations
- `math`: Mathematical functions
- `fs`: File system operations
- `time`: Time and date functions

No need to import builtins:

```loft
term.println("Hello");  // Works directly
```
