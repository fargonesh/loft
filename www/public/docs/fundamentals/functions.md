# Functions

## Declaring Functions

Functions are declared with the `fn` keyword:

```loft
fn greet() {
    term.println("Hello!");
}

greet();  // Call the function
```

## Parameters

Functions can accept parameters:

```loft
fn greet(name: str) {
    term.println("Hello, ");
    term.println(name);
}

greet("Alice");
```

Multiple parameters:

```loft
fn add(a: num, b: num) {
    return a + b;
}

let result = add(5, 3);
term.println(result);  // 8
```

## Return Values

Use return type annotation after `->`:

```loft
fn add(a: num, b: num) -> num {
    return a + b;
}
```

The `return` keyword explicitly returns a value:

```loft
fn max(a: num, b: num) -> num {
    if a > b {
        return a;
    }
    return b;
}
```

## Implicit Returns

The last expression in a function is returned implicitly:

```loft
fn add(a: num, b: num) -> num {
    a + b  // No semicolon, implicitly returned
}
```

## Optional Return Types

Functions without return type annotations return void:

```loft
fn log_message(msg: str) {
    term.println(msg);
}  // Returns void implicitly
```

## Function Scope

Functions can access variables from outer scopes:

```loft
let multiplier = 2;

fn multiply(x: num) -> num {
    return x * multiplier;
}

term.println(multiply(5));  // 10
```

## Async Functions

Declare asynchronous functions with `async`:

```loft
async fn fetch_data() -> str {
    // Async operations here
    return "data";
}

// Call async functions with await
async fn main() {
    let data = await fetch_data();
    term.println(data);
}
```

## Higher-Order Functions

Functions can accept other functions as parameters:

```loft
fn apply(f: any, x: num) -> num {
    return f(x);
}

fn double(x: num) -> num {
    return x * 2;
}

let result = apply(double, 5);
term.println(result);  // 10
```
