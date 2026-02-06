---
name: fix-test-failures
description: "This skill should be used after running tests when failures occur. It ensures test failures are properly diagnosed through instrumentation and logging until the root cause is found and fixed. The skill treats all test failures as real bugs that must be resolved, never skipped."
---

# Fix Test Failures

This skill provides a systematic approach to diagnosing and fixing test failures. The core philosophy is that **test failures are real bugs** - they must be understood and fixed, never ignored or worked around.

## NO CHEATING - Critical Tests

**These tests validate exact behavioral matching with the C++ Clipper2 implementation. There are no workarounds.**

Every test exists because it validates behavior that must match C++ exactly. Bypassing tests means shipping incorrect implementations.

**Forbidden actions (no exceptions):**
- Weakening assertions to make tests pass
- Changing expected values to match broken behavior
- Wrapping failing code in catch blocks to swallow errors
- Adding conditional logic to skip checks in test environments
- Commenting out assertions or test blocks
- Using `todo!()` or `unimplemented!()` to defer failures
- Relaxing precision requirements to mask numerical errors
- Mocking away the actual behavior being tested

**The only acceptable outcome is fixing the actual bug in the production code.**

## When to Use This Skill

Use this skill when:
- Tests fail and the cause isn't immediately obvious
- A test is flaky or intermittently failing
- You need to understand why a test is failing before fixing it
- You've made changes and tests are now failing

## Core Principles

1. **Test failures are real bugs** - Never skip, disable, or delete failing tests without understanding and fixing the underlying issue
2. **No cheating** - Never weaken tests, change expected values, or work around failures
3. **Instrument to understand** - Add print statements to expose internal state and execution flow
4. **Fix the root cause** - Don't patch symptoms; find and fix the actual bug
5. **Clean up after** - Remove instrumentation once the fix is verified

## Test Failure Resolution Process

### Step 1: Run Tests and Capture Failures

Run the failing test(s) to see the current error:

```bash
# Run all tests
cargo test

# Run tests in a specific module
cargo test --lib engine_tests

# Run a specific test
cargo test test_name -- --exact

# Run with output visible
cargo test test_name -- --nocapture

# Run with backtrace on failure
RUST_BACKTRACE=1 cargo test test_name -- --nocapture
```

Record the exact error message and stack trace. This is your starting point.

### Step 2: Analyze the Failure

Before adding instrumentation, understand what the test is checking:

1. Read the test code carefully
2. Identify what assertion is failing
3. Note what values were expected vs. received
4. Cross-reference with the C++ implementation to verify expected behavior
5. Form a hypothesis about what might be wrong

### Step 3: Add Strategic Instrumentation

Add `println!` or `dbg!` statements to expose the state at key points. Target areas include:

**For state-related failures:**
```rust
println!("State before operation: {:?}", state);
// ... operation ...
println!("State after operation: {:?}", state);
```

**For numerical precision failures:**
```rust
println!("Expected: {:.15}", expected);
println!("Actual:   {:.15}", actual);
println!("Diff:     {:.15e}", (expected - actual).abs());
```

**For path/polygon failures:**
```rust
println!("Path length: {}", path.len());
println!("First point: {:?}", path.first());
println!("Last point: {:?}", path.last());
println!("Area: {}", area(&path));
```

**For function execution flow:**
```rust
println!("Entering function with args: {:?}, {:?}", arg1, arg2);
// ... function body ...
println!("Returning result: {:?}", result);
```

**Using dbg! for quick inspection:**
```rust
let result = dbg!(compute_intersection(pt1, pt2, pt3, pt4));
```

### Step 4: Run Instrumented Tests

Run the test again with output visible:

```bash
cargo test test_name -- --nocapture
```

Analyze the output to understand:
- What values are actually present
- Where the execution diverges from expectations
- What state is incorrect and when it became incorrect

### Step 5: Identify Root Cause

Based on instrumentation output, determine:
- Is the test wrong (rare - only if test assumptions were incorrect)?
- Is the code under test wrong (common)?
- Is there a numerical precision issue?
- Is there an issue with type conversion (i64 vs f64, etc.)?
- Does the Rust implementation diverge from C++ behavior?

### Step 6: Fix the Bug

Fix the actual bug in the production code, not by modifying the test to accept wrong behavior.

Common fixes for this project:
- **Algorithm errors**: Verify against C++ implementation step-by-step
- **Precision issues**: Check for integer overflow, floating point order of operations
- **Type mismatches**: Ensure proper conversions between integer and float coordinates
- **Edge cases**: Verify handling of collinear points, empty paths, degenerate polygons
- **Winding rules**: Verify correct application of EvenOdd, NonZero, Positive, Negative

### Step 7: Verify and Clean Up

1. Run the test again to confirm it passes
2. Run the full test suite to ensure no regressions: `cargo test`
3. **Remove all instrumentation print statements** - they were for debugging only
4. Commit the fix

## Iterative Debugging

If the first round of instrumentation doesn't reveal the issue:

1. Add more instrumentation at earlier points in execution
2. Log intermediate values, not just final state
3. Compare intermediate values with C++ execution (run C++ tests for the same input)
4. Check for side effects from other code
5. Verify test setup is correct

Keep iterating until the root cause is clear. The goal is understanding, then fixing.

## C++ Comparison Technique

When stuck, compare execution step-by-step with C++:

1. Add corresponding print statements in both Rust and C++ code
2. Run both with the same input
3. Find the first point of divergence
4. That divergence point is your bug location

## What NOT to Do (NO CHEATING)

These are all forms of cheating that bypass the purpose of testing:

- **Don't skip failing tests** - Every test failure is meaningful
- **Don't delete tests** to make the suite pass
- **Don't use `#[ignore]`** as a permanent solution
- **Don't weaken assertions** - If a test expects 3, don't change it to expect 2
- **Don't change expected values** to match broken output
- **Don't use `todo!()`** to defer the fix
- **Don't relax precision** to hide numerical errors
- **Don't leave instrumentation** in committed code

**If you find yourself wanting to do any of these, STOP. The test is telling you something is broken. Fix the broken thing.**
