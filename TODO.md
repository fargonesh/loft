# loft Language Development TODO

Last Updated: December 19, 2024

## Current Status

Phase 1 Core Features: COMPLETE
- Closures and lambdas: Working
- Enums and pattern matching: Working
- Module system: Working
- Error propagation: Working
- Test coverage: 87% (140 tests passing)
- Documentation: Complete (7 files)
- Parser improvements: Optional return types implemented

Launch Readiness: 100% (11/11 criteria met)

## Immediate Pre-Launch Tasks

### 1. Documentation Consolidation
- Create comprehensive book using mdBook
- Rewrite all book chapters with clear examples
- Ensure stdlib documentation is accessible and accurate
- Add getting started guide
- Create language tour
- Add API reference for all builtins

### 2. Build and Distribution
- Create GitHub Actions workflow for automated builds
- Build binaries for Linux, macOS, and Windows
- Create release artifacts with proper naming
- Update install.sh script to download from GitHub releases
- Test installation on multiple platforms
- Add version management to binaries

### 3. Testing and Quality
- Verify all 140 tests pass
- Run tests on CI
- Add integration tests for real-world scenarios
- Test example projects
- Verify LSP functionality

## Phase 2: Developer Experience (Post-Launch)

### Enhanced LSP Features
- Implement auto-import code actions
- Add quick fixes for common errors
- Improve cross-file go-to-definition
- Add find references across workspace
- Implement rename refactoring (cross-file)
- Add call hierarchy (incoming/outgoing calls)
- Improve completion ranking and context
- Add postfix completions (.if, .match, etc.)

### Standard Library Expansion
- Add JSON parsing and serialization
- Implement HTTP client builtin
- Add process/command execution
- Environment variable access
- Crypto/hashing utilities
- Regex support
- Date/time formatting utilities
- Collections (HashMap, Set)
- String interpolation syntax

## Phase 3: Production Hardening

### Testing and Quality
- Expand test coverage to 90%+
- Add integration test suite
- Create example projects:
  - CLI tool
  - Web scraper
  - REST API server
- Performance profiling and optimization
- Fix memory leaks if any
- Stress test with large files
- Add benchmarks for common operations
- Audit dependencies for security

### Documentation Expansion
- Complete missing documentation pages
- Write "Getting Started" tutorial
- Create "Language Tour" guide
- Add recipe book with common patterns
- Document best practices
- Add migration guides from other languages
- Create cheat sheet/quick reference
- Set up documentation website
- Add FAQ section

### Community Preparation
- Set up issue templates
- Create PR template
- Improve CI/CD pipeline
- Add automated release process
- Write announcement blog post
- Prepare demo video
- Set up Discord or community forum
- Create social media presence

## Phase 4: Advanced Features (Post-Launch)

### Type System Improvements
- Implement static type checking pass
- Check function call arity and types
- Validate binary operation types
- Check struct field types
- Improve type inference
- Add type mismatch diagnostics in LSP
- Support union types

### Language Features
- Full generics implementation
- Trait bounds and where clauses
- Advanced pattern matching (guards, ranges)
- Macros and metaprogramming
- Procedural macros
- Unsafe blocks for FFI
- Native library binding system

### Performance
- Compile to bytecode
- JIT compilation
- WASM compilation target
- Incremental compilation
- Memory optimization
- Parallel execution improvements

### Tooling
- Interactive debugger
- Profiler integration
- Package discovery website
- Online playground/IDE
- Enhanced formatter options
- Linter with custom rules

## Known Issues and Limitations

### Resolved
- Parser edge cases with function signatures: FIXED (optional return types)
- All critical bugs: RESOLVED

### Future Work
- Performance benchmarks not yet defined
- Array methods with closures (map, filter, reduce)
- Mutable variable captures in closures
- Advanced async/await patterns
- Generic trait implementations
- Const generics

## Technical Debt

### Parser
- Improve error recovery
- Better error messages with suggestions
- Support for more expression contexts
- Optimize parsing performance

### Runtime
- Memory management improvements
- Garbage collection strategy
- Stack overflow protection
- Better async runtime

### LSP
- Optimize for large files
- Incremental parsing
- Better semantic tokens
- Workspace symbol caching

## Decision Points

### Phase 2 Decisions
- Auto-import behavior: top of file vs inline
- Completion triggers: aggressive vs conservative
- HTTP API design: Fetch-like vs Axios-like vs custom
- JSON handling: typed parsing vs dynamic
- String interpolation syntax: ${} vs {}

### Phase 3 Decisions
- Type checking strictness: strict mode vs always on
- Gradual typing: allow mixing typed and untyped code
- Type inference: infer return types vs require explicit annotations

### Phase 4 Decisions
- Macro syntax and capabilities
- FFI interface design
- Compilation target priorities
- Backward compatibility policy

## Release Checklist

### Before Launch
- All tests passing
- Documentation complete
- Install script working
- GitHub Actions building artifacts
- Example projects working
- README up to date
- CHANGELOG complete
- License file present
- Contributing guidelines clear

### Launch Day
- Tag release in git
- Publish GitHub release with binaries
- Update website if applicable
- Post announcement
- Monitor for issues
- Respond to early feedback

### Post-Launch
- Address bug reports promptly
- Collect feature requests
- Plan next release cycle
- Update roadmap based on feedback
- Build community engagement
