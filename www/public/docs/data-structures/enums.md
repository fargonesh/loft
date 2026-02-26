# Enums

Enums represent values that can be one of several variants.

## Unit Variants

Enums with simple variants:

```loft
enum Status {
    Pending,
    Active,
    Inactive,
}

let status = Status.Active;
```

## Tuple Variants

Variants can hold data:

```loft
enum Result {
    Ok(num),
    Err(str),
}

let success = Result.Ok(42);
let failure = Result.Err("not found");
```

Multiple values:

```loft
enum Message {
    Move(num, num),
    Write(str),
    ChangeColor(num, num, num),
}

let msg = Message.Move(10, 20);
```

## Pattern Matching

Use match to handle variants:

```loft
enum Result {
    Ok(num),
    Err(str),
}

let result = Result.Ok(42);
match result {
    Result.Ok(value) => {
        term.println("Success:");
        term.println(value);
    },
    Result.Err(msg) => {
        term.println("Error:");
        term.println(msg);
    },
};
```

## Common Patterns

### Option Type

Represent optional values:

```loft
enum Option {
    Some(num),
    None,
}

fn find(arr: any, target: num) -> Option {
    for item in arr {
        if item == target {
            return Option.Some(item);
        }
    }
    return Option.None;
}
```

### Result Type

Represent operations that can fail:

```loft
enum Result {
    Ok(str),
    Err(str),
}

fn read_file(path: str) -> Result {
    // Try to read file
    return Result.Ok("file contents");
    // Or return Result.Err("file not found");
}
```
