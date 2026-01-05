# loft Language Development TODO

Last Updated: December 20, 2024

### Future Work
- Performance benchmarks not yet defined
- Advanced async/await patterns
- Generic trait implementations
- Const generics

## Technical Debt

### Parser
- Improve error recovery
- Support for more expression contexts
- Optimize parsing performance

### Runtime
- Memory management improvements
- Garbage collection strategy
- Better async runtime

### LSP
- Optimize for large files
- Incremental parsing
- Better semantic tokens
- Workspace symbol caching

## Decision Points

### Phase 2 Decisions
- Auto-import behavior: top of file vs inline (Top of file)
- Completion triggers: aggressive vs conservative (Aggressive)
- HTTP API design: Fetch-like vs Axios-like vs custom (Fetch-like)
- JSON handling: typed parsing vs dynamic (Typed, .parse<T>())
- String interpolation syntax: ${} vs {} (${})

### Phase 3 Decisions
- Type checking strictness: strict mode vs always on (Strict)
- Gradual typing: allow mixing typed and untyped code (No mixing, all explicit or implicit)
- Type inference: infer return types vs require explicit annotations (require explicit)

### Phase 4 Decisions
- Macro syntax and capabilities (rust proc-macro-like, see quote. syn.)
- FFI interface design
- Backward compatibility policy (every major version may impose breaking backwards changes, to be readressed after language reaches maturity, maybe backwards compat to the nearest lts)