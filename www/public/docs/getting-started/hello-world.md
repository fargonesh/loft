# Hello World

Let's write your first loft program!

## Your First Program

Create a file named `hello.lf`:

```loft
term.println("Hello, World!");
```

Run it:

```bash
loft hello.lf
```

You should see:
```
Hello, World!
```

## Understanding the Code

- `term` is a builtin module for terminal input/output
- `println` is a method that prints text followed by a newline
- Strings are enclosed in double quotes

## Adding Variables

Let's make it more interesting:

```loft
let name = "Alice";
term.println("Hello, ");
term.println(name);
term.println("!");
```

Or combine it:

```loft
let name = "Alice";
let greeting = "Hello, " + name + "!";
term.println(greeting);
```

## Functions

Create a reusable greeting function:

```loft
fn greet(name: str) {
    term.println("Hello, ");
    term.println(name);
    term.println("!");
}

greet("Alice");
greet("Bob");
```

## Comments

Add comments to explain your code:

```loft
// This is a single-line comment

// Functions can have multiple parameters
fn greet(greeting: str, name: str) {
    term.println(greeting);
    term.println(name);
}

greet("Hello", "World");
```

## Next Steps

Now that you've written your first program, learn about [Basic Syntax](./syntax.md) to understand loft's structure.
