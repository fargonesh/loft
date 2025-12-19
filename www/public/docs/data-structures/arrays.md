# Arrays

Arrays store ordered collections of values.

## Creating Arrays

Array literals use square brackets:

```loft
let numbers = [1, 2, 3, 4, 5];
let names = ["Alice", "Bob", "Carol"];
let empty = [];
```

## Accessing Elements

Use index notation (0-based):

```loft
let numbers = [10, 20, 30, 40];
term.println(numbers[0]);   // 10
term.println(numbers[1]);   // 20
term.println(numbers[3]);   // 40
```

## Array Methods

The `array` module provides utility functions:

```loft
let numbers = [1, 2, 3, 4, 5];

// Get length
let len = array.len(numbers);
term.println(len);  // 5

// Push element
array.push(numbers, 6);

// Pop element
let last = array.pop(numbers);
term.println(last);  // 6
```

## Iteration

Use for loops to iterate:

```loft
let numbers = [1, 2, 3, 4, 5];
for num in numbers {
    term.println(num);
}
```

## Mixed Types

Arrays can hold different types:

```loft
let mixed = [1, "two", 3.0, true];
```
