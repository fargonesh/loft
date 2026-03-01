# loft Language Development TODO

Last Updated: March 1, 2026

## ✅ Completed Items
- **Parser Error Recovery**: Basic `synchronize` mechanism implemented for statement-level recovery.
- **Async/Await**: Basic `async` and `await` keywords implemented with `Promise` value type.
- **Semantic Tokens**: Basic semantic tokens legend and provider implemented in LSP.
- **FFI Interface**: Support for loading shared libraries and calling functions with numeric arguments.
- **String Interpolation**: `${}` syntax implemented in `TokenStream`.
- **JSON Handling**: basic `json.parse()` and `json.stringify()` (via `to_string`) implemented.

## 🚀 Future Work
- **Performance Benchmarks**: Not yet defined or automated.
- **Advanced Async/Await**: 
    - Real task scheduling (currently synchronous simulation).
    - True lazy futures (currently eager).
- **Generic Trait Implementations**: Parser supports them, but runtime lacks full implementation for generic types.
- **Const Generics**: Initial parser support for `const` declarations, but no generic constraints yet.
- **Memory Management**: 
    - Move from `Clone` heavy semantics to a more efficient strategy (e.g., Reference Counting or GC).
    - Current `Value` type uses significant cloning.

## 🛠️ Technical Debt

### Parser
- **Attribute System**: Needs more robust handling and validation beyond gated features.
- **Incremental Parsing**: LSP currently re-parses full files.

### Runtime
- **Type Safety**: Runtime currently skips much of the type validation even when annotations are present.
- **Module System**: Improve isolation and circular dependency handling.

### LSP
- **Workspace Symbol Caching**: Currently missing, leading to full scans.
- **Go to Definition**: Expand beyond basic types and functions to trait methods and cross-module symbols.

## ⚖️ Decision Points Status

### Phase 2-4 Decisions (Carry-over/Ongoing)
- **Auto-import behavior**: Finalize convention for module-level vs explicit imports.
- **Backward Compatibility**: Define the exact LTS policy once 1.0 is reached.
- **Macro Capabilities**: Expand `loft_builtin_macros` to support user-defined procedural macros.
