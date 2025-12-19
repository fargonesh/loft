# Error Handling

loft uses explicit error handling with Result types and the error propagation operator.

## Result Type

Represent operations that can succeed or fail:

```loft
enum Result {
    Ok(num),
    Err(str),
}

fn divide(a: num, b: num) -> Result {
    if b == 0 {
        return Result.Err("Division by zero");
    }
    return Result.Ok(a / b);
}
```

## Pattern Matching

Handle success and failure cases:

```loft
let result = divide(10, 2);
match result {
    Result.Ok(value) => {
        term.println("Result: ");
        term.println(value);
    },
    Result.Err(msg) => {
        term.println("Error: ");
        term.println(msg);
    },
};
```

## Error Propagation Operator

The `?` operator propagates errors up the call stack:

```loft
fn process() -> Result {
    let a = divide(10, 2)?;  // If error, return immediately
    let b = divide(a, 2)?;
    return Result.Ok(b);
}
```

This is equivalent to:

```loft
fn process() -> Result {
    let a_result = divide(10, 2);
    let a = match a_result {
        Result.Ok(v) => v,
        Result.Err(e) => return Result.Err(e),
    };
    
    let b_result = divide(a, 2);
    let b = match b_result {
        Result.Ok(v) => v,
        Result.Err(e) => return Result.Err(e),
    };
    
    return Result.Ok(b);
}
```

## Option Type

Represent optional values:

```loft
enum Option {
    Some(num),
    None,
}

fn find_index(arr: any, target: num) -> Option {
    let i = 0;
    for item in arr {
        if item == target {
            return Option.Some(i);
        }
        i = i + 1;
    }
    return Option.None;
}
```

## Best Practices

Return Result for operations that can fail:

```loft
fn read_config(path: str) -> Result {
    // Read file, parse config
    return Result.Ok("config data");
}
```

Use Option for values that might not exist:

```loft
fn get_user(id: num) -> Option {
    // Query database
    return Option.Some("user data");
    // Or Option.None if not found
}
```
