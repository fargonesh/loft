# Variables and Constants

## Variables

Declare variables with the `let` keyword:

```loft
let x = 5;
let name = "Alice";
let is_active = true;
```

Variables are immutable by default. You cannot reassign them:

```loft
let x = 5;
x = 10;  // Error: cannot reassign immutable variable
```

## Type Annotations

You can optionally specify types:

```loft
let x: num = 5;
let name: str = "Alice";
let is_active: bool = true;
```

Type annotations are checked at runtime.

## Constants

Constants are declared with `const` and must have a value:

```loft
const PI = 3.14159;
const MAX_SIZE = 100;
```

Constants cannot be reassigned and are available throughout their scope.

## Scope

Variables are scoped to the block they're declared in:

```loft
let x = 5;
{
    let y = 10;
    term.println(x);  // OK: x is accessible
    term.println(y);  // OK: y is accessible
}
term.println(y);  // Error: y is out of scope
```

## Shadowing

You can declare a new variable with the same name, which shadows the previous one:

```loft
let x = 5;
term.println(x);  // Prints 5

let x = x + 10;
term.println(x);  // Prints 15

let x = "now a string";
term.println(x);  // Prints "now a string"
```

Shadowing creates a new variable, so you can change the type.
