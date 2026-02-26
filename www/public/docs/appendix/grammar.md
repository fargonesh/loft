# Language Grammar

High-level grammar overview.

## Declarations
- Function: `fn name(params) -> type { body }`
- Struct: `def Name { fields }`
- Enum: `enum Name { variants }`
- Variable: `let name = value`
- Constant: `const NAME = value`

## Expressions
- Literals: `42`, `"string"`, `true`, `[1,2,3]`
- Binary ops: `+`, `-`, `*`, `/`, `==`, `!=`, `<`, `>`, `<=`, `>=`
- Logical ops: `&&`, `||`, `!`
- Function call: `func(args)`
- Lambda: `x => expr` or `(x, y) => expr`
- Match: `match value { pattern => expr }`
