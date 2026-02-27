# Documentation Generator

Generate HTML documentation from loft code.

## Usage

Generate docs:
```bash
loft doc
```

Output is written to the `docs/` directory.

## Doc Comments

Add documentation with comments:
```loft
// Calculate the sum of two numbers
fn add(a: num, b: num) -> num {
    return a + b;
}
```
