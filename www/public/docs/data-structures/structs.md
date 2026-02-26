# Structs

Structs group related data together.

## Defining Structs

Use the `def` keyword:

```loft
def Person {
    name: str,
    age: num,
}
```

## Creating Instances

Provide values for all fields:

```loft
let person = Person {
    name: "Alice",
    age: 30,
};
```

## Accessing Fields

Use dot notation:

```loft
term.println(person.name);  // Alice
term.println(person.age);   // 30
```

## Methods

Define methods in impl blocks:

```loft
def Rectangle {
    width: num,
    height: num,
}

impl Rectangle {
    fn area(self) -> num {
        return self.width * self.height;
    }
    
    fn perimeter(self) -> num {
        return 2 * (self.width + self.height);
    }
}

let rect = Rectangle { width: 10, height: 20 };
term.println(rect.area());       // 200
term.println(rect.perimeter());  // 60
```

## Nested Structs

Structs can contain other structs:

```loft
def Address {
    street: str,
    city: str,
}

def Person {
    name: str,
    address: Address,
}

let person = Person {
    name: "Alice",
    address: Address {
        street: "123 Main St",
        city: "Springfield",
    },
};

term.println(person.address.city);  // Springfield
```
