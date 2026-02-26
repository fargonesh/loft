# Basic Syntax

## Comments

loft supports single-line comments:

```loft
// This is a comment
let x = 5;  // Comments can follow code
```

## Statements and Expressions

In loft, most things are expressions that return values:

```loft
let x = 5;              // Statement
let y = x + 10;         // Expression
let result = if x > 0 { "positive" } else { "negative" };  // Expression
```

## Semicolons

Semicolons are optional at the end of lines but required to separate multiple statements on one line:

```loft
let x = 5;
let y = 10;

// Multiple statements on one line need semicolons
let a = 1; let b = 2;
```

## Blocks

Code blocks are enclosed in curly braces:

```loft
{
    let x = 5;
    let y = 10;
    term.println(x + y);
}
```

The last expression in a block is its return value:

```loft
let result = {
    let x = 5;
    let y = 10;
    x + y  // No semicolon means this is returned
};
term.println(result);  // Prints 15
```

## Identifiers

Identifiers (variable names, function names) must:
- Start with a letter or underscore
- Contain only letters, numbers, and underscores
- Not be a keyword

Valid identifiers:
```loft
my_variable
count2
_private
camelCase
