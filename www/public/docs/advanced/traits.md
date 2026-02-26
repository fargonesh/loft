# Traits

Traits define shared behavior that types can implement.

## Builtin Traits

loft includes several builtin traits:

```loft
trait Add {
    fn add(self, other: Self) -> Self;
}

trait ToString {
    fn to_string(self) -> str;
}

trait Printable {
    fn print(self);
}
```

## Implementing Traits

Use impl blocks to implement traits:

```loft
def Point {
    x: num,
    y: num,
}

impl Add for Point {
    fn add(self, other: Point) -> Point {
        return Point {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

let p1 = Point { x: 1, y: 2 };
let p2 = Point { x: 3, y: 4 };
let p3 = p1.add(p2);
```

## Operator Overloading

Traits enable operator overloading:

```loft
impl Add for Point {
    fn add(self, other: Point) -> Point {
        return Point {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

let p1 = Point { x: 1, y: 2 };
let p2 = Point { x: 3, y: 4 };
let p3 = p1 + p2;  // Uses Add trait
```
