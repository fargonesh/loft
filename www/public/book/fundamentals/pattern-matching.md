# Pattern Matching

Pattern matching with `match` is a powerful way to handle different cases.

## Basic Matching

Match on enum variants:

```loft
enum Color {
    Red,
    Green,
    Blue,
}

let color = Color.Red;
match color {
    Color.Red => term.println("Red"),
    Color.Green => term.println("Green"),
    Color.Blue => term.println("Blue"),
};
```

## Destructuring

Extract values from enum variants:

```loft
enum Message {
    Text(str),
    Number(num),
}

let msg = Message.Text("Hello");
match msg {
    Message.Text(content) => term.println(content),
    Message.Number(value) => term.println(value),
};
```

Multiple values:

```loft
enum Point {
    TwoD(num, num),
    ThreeD(num, num, num),
}

let point = Point.TwoD(10, 20);
match point {
    Point.TwoD(x, y) => {
        term.println("2D point:");
        term.println(x);
        term.println(y);
    },
    Point.ThreeD(x, y, z) => {
        term.println("3D point:");
        term.println(x);
        term.println(y);
        term.println(z);
    },
};
```

## Match Expressions

Match returns a value:

```loft
enum Status {
    Ok(num),
    Err(str),
}

let status = Status.Ok(200);
let message = match status {
    Status.Ok(code) => "Success: " + code,
    Status.Err(msg) => "Error: " + msg,
};
term.println(message);
```

## Nested Matching

Match on nested structures:

```loft
enum Option {
    Some(num),
    None,
}

enum Result {
    Ok(Option),
    Err(str),
}

let result = Result.Ok(Option.Some(42));
match result {
    Result.Ok(Option.Some(value)) => term.println(value),
    Result.Ok(Option.None) => term.println("No value"),
    Result.Err(msg) => term.println(msg),
};
```
