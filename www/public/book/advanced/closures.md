# Closures and Lambdas

Closures are anonymous functions that can capture variables from their environment.

## Lambda Syntax

Simple lambda with one parameter:

```loft
let double = x => x * 2;
term.println(double(5));  // 10
```

Multiple parameters require parentheses:

```loft
let add = (a, b) => a + b;
term.println(add(3, 4));  // 7
```

With type annotations:

```loft
let multiply = (x: num, y: num) => x * y;
term.println(multiply(3, 4));  // 12
```

## Capturing Environment

Closures can access variables from outer scopes:

```loft
let multiplier = 10;
let scale = x => x * multiplier;
term.println(scale(5));  // 50
```

Multiple captures:

```loft
let prefix = "Value: ";
let suffix = "!";
let format = x => prefix + x + suffix;
term.println(format("42"));  // Value: 42!
```

## Closures as Parameters

Pass closures to functions:

```loft
fn apply(f: any, x: num) -> num {
    return f(x);
}

let result = apply(x => x * 2, 5);
term.println(result);  // 10
```

## Block Bodies

Closures can have block bodies for multiple statements:

```loft
let process = (x: num) => {
    let doubled = x * 2;
    let squared = doubled * doubled;
    squared
};

term.println(process(3));  // 36
```

## Practical Examples

Filter and transform data:

```loft
fn transform(arr: any, func: any) -> any {
    let result = [];
    for item in arr {
        array.push(result, func(item));
    }
    return result;
}

let numbers = [1, 2, 3, 4, 5];
let doubled = transform(numbers, x => x * 2);
// doubled is [2, 4, 6, 8, 10]
```
