# Claude Code Guidelines

## Philosophy

**Quality through iterations** - Start with correct implementations, then improve. Code that doesn't matter can be quick and dirty. But code that matters *really* matters—treat it with respect and improve it meticulously. In a porting project, every function matters.

**Circumstances alter cases** - Use judgment. There are no rigid rules—context determines the right approach. However, this project has strong defaults because porting demands precision.

**No stubs, no shortcuts** - Every function must be complete and production-ready. No `todo!()`, no `unimplemented!()`, no `panic!("not implemented")`, no partial implementations. If dependencies aren't ready, stop and implement them first.

## Test-First Bug Fixing (Critical Practice)

**This is the single most important practice for agent performance and reliability.**

When a bug is reported, always follow this workflow:

1. **Write a reproducing test first** - Create a test that fails, demonstrating the bug
2. **Fix the bug** - Make the minimal change needed to address the issue
3. **Verify via passing test** - The previously failing test should now pass

This approach works because:
- The failing test proves you understand the bug
- The fix is verifiable, not just "looks right"
- You can't accidentally break it again (regression protection)
- It aligns with the principle that coding is high-leverage because it's **partially verifiable**

**Do not skip the reproducing test.** Even if the fix seems obvious, the test validates your understanding and prevents regressions.

## Testing

- Tests MUST test actual production code, not copies - Never duplicate production logic in tests. Import and call the real code. Tests that verify copied code prove nothing about the actual system.
- Tests should run as fast as possible—fast tests get run more often
- Write tests for regressions and complex logic
- Avoid redundant tests that verify the same behavior
- All tests must pass before merging
- Tests must verify **exact behavioral match** with the C++ implementation
- When test failures occur, use the fix-test-failures agent (`.claude/agents/fix-test-failures.md`) — it treats all failures as real bugs and resolves them through instrumentation and root cause analysis, never by weakening tests

**Running tests:**
```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test --lib core_tests
cargo test --lib engine_tests

# Run a specific test
cargo test test_name -- --exact

# Run with output
cargo test -- --nocapture
```

## Code Quality

**Names** - Choose carefully. Good names make code self-documenting. Rust names should follow Rust conventions (`snake_case` for functions/variables, `PascalCase` for types, `SCREAMING_SNAKE_CASE` for constants).

**Comments** - Explain *why*, not *what*. The code shows what it does; comments should reveal intent, tradeoffs, and non-obvious reasoning. When porting from C++, comments explaining *why* the Rust approach differs from C++ are especially valuable.

**Refactoring** - Improve code when it serves a purpose, not for aesthetics. Refactor to fix bugs, add features, or improve clarity when you're already working in that area.

## C++ to Rust Porting Rules

This project is a strict port of the Clipper2 C++ library to Rust. These rules ensure fidelity:

### Exact Behavioral Matching
- Rust implementation must match C++ behavior exactly
- Same algorithms, same mathematical precision
- Same edge case handling, same error conditions
- Same performance characteristics (or better)
- No "close enough" implementations

### Dependency-Ordered Implementation
Before implementing any function:
1. Read the corresponding C++ source to identify all functions called by the target function
2. Verify all dependencies are already implemented and tested in the Rust codebase
3. If any dependency is incomplete, implement dependencies first

### Forbidden Patterns
- `todo!()` or `unimplemented!()` macros
- `panic!()` for missing functionality
- Stub functions or placeholder implementations
- Implementing without dependencies ready
- Marking functions complete prematurely
- "Close enough" or "good enough for now" implementations

## C++ Reference

Original C++ source code is in the `CPP/` directory for reference during porting.
