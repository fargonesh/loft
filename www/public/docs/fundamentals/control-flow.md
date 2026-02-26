# Control Flow

## If Expressions

Basic if statements:

```loft
let x = 5;
if x > 0 {
    term.println("positive");
}
```

With else:

```loft
let x = -3;
if x > 0 {
    term.println("positive");
} else {
    term.println("not positive");
}
```

Else if chains:

```loft
let x = 0;
if x > 0 {
    term.println("positive");
} else if x < 0 {
    term.println("negative");
} else {
    term.println("zero");
}
```

## If as Expression

If expressions return values:

```loft
let x = 5;
let message = if x > 0 { "positive" } else { "not positive" };
term.println(message);
```

## While Loops

Repeat while a condition is true:

```loft
let i = 0;
while i < 5 {
    term.println(i);
    i = i + 1;
}
```

## For Loops

Iterate over arrays:

```loft
let numbers = [1, 2, 3, 4, 5];
for num in numbers {
    term.println(num);
}
```

## Match Expressions

Pattern match on values:

```loft
enum Status {
    Pending,
    Active,
    Inactive,
}

let status = Status.Active;
let message = match status {
    Status.Pending => "Waiting",
    Status.Active => "Running",
    Status.Inactive => "Stopped",
};
term.println(message);  // Running
```

Match with data extraction:

```loft
enum Result {
    Ok(num),
    Err(str),
}

let result = Result.Ok(42);
match result {
    Result.Ok(value) => term.println(value),
    Result.Err(msg) => term.println(msg),
};
```

## Return and Break

Early return from functions:

```loft
fn find(arr: any, target: num) -> num {
    for item in arr {
        if item == target {
            return Result.Ok(item);
        }
    }
    return Result.Err("not found");
}
```
