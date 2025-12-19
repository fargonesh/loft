# Terminal I/O (term)

The `term` module provides terminal input and output functions.

## Functions

### println(value: any)

Print a value followed by a newline:

```loft
term.println("Hello, World!");
term.println(42);
term.println(true);
```

### print(value: any)

Print a value without a newline:

```loft
term.print("Hello, ");
term.print("World!");
// Output: Hello, World!
```

### read() -> str

Read a line from standard input:

```loft
term.print("Enter your name: ");
let name = term.read();
term.println("Hello, " + name);
```
