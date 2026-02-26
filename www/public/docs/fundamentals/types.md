# Data Types

loft has several built-in data types.

## Numeric Types

The `num` type represents both integers and decimals:

```loft
let integer = 42;
let decimal = 3.14159;
let negative = -10;
let scientific = 1.5e-10;
```

## Strings

Strings are UTF-8 encoded text enclosed in double quotes:

```loft
let greeting = "Hello, World!";
let emoji = "Hello 👋";
let empty = "";
```

String concatenation uses the `+` operator:

```loft
let first = "Hello";
let second = "World";
let combined = first + " " + second;
```

## Booleans

The `bool` type has two values:

```loft
let is_true = true;
let is_false = false;
```

Boolean operators:

```loft
let and_result = true && false;   // false
let or_result = true || false;    // true
let not_result = !true;           // false
```

## Arrays

Arrays hold ordered collections of values:

```loft
let numbers = [1, 2, 3, 4, 5];
let names = ["Alice", "Bob", "Carol"];
let mixed = [1, "two", 3.0];  // Arrays can hold different types
```

Access elements by index (0-based):

```loft
let numbers = [10, 20, 30];
let first = numbers[0];   // 10
let second = numbers[1];  // 20
```

## Structs

Structs group related data:

```loft
def Person {
    name: str,
    age: num,
}

let person = Person {
    name: "Alice",
    age: 30,
};

term.println(person.name);  // Alice
term.println(person.age);   // 30
```

## Enums

Enums represent a value that can be one of several variants:

```loft
enum Status {
    Pending,
    Active,
    Inactive,
}

let status = Status.Active;
```

Enums can hold data:

```loft
enum Result {
    Ok(num),
    Err(str),
}

let success = Result.Ok(42);
let failure = Result.Err("not found");
```

## Unit Type

The unit type (void) represents the absence of a value:

```loft
fn do_something() {
    term.println("Done");
}  // Implicitly returns unit
```

## Type Checking

loft performs runtime type checking:

```loft
let x: num = "hello";  // Runtime error: type mismatch
```

Type annotations are optional but help catch errors:

```loft
fn add(a: num, b: num) -> num {
    return a + b;
}

add(5, "10");  // Runtime error: expected num, got str
```
