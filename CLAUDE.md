# CLIPPER2 RUST PORT - STRICT IMPLEMENTATION RULES

## ZERO TOLERANCE POLICY

This project follows a **ZERO TOLERANCE POLICY** for incomplete implementations. Every rule below is **MANDATORY** and **NON-NEGOTIABLE**.

### RULE 1: COMPLETE FUNCTIONS ONLY
- **NO** `todo!()` macros allowed
- **NO** `unimplemented!()` macros allowed  
- **NO** `panic!("not implemented")` allowed
- **NO** stub functions or placeholder implementations
- **NO** partial implementations that "work for basic cases"
- EVERY function must be **COMPLETE** and **PRODUCTION-READY**

### RULE 2: STRICT DEPENDENCY ENFORCEMENT
- Functions **CANNOT** be implemented if their dependencies are incomplete
- Implementation **MUST WAIT** until ALL dependency functions are:
  - Fully implemented
  - Fully tested
  - Marked as complete in database
- Implementation order **MUST** follow dependency chain strictly
- **NO EXCEPTIONS** - even if "it would be easy to stub this one call"

### RULE 3: MANDATORY COMPREHENSIVE TESTING  
- **EVERY** function must have unit tests before marking as complete
- Tests must cover **ALL** edge cases and error conditions
- Tests must verify **EXACT** behavioral match with C++ implementation
- **NO** function is considered "complete" until tests pass 100%
- Test coverage must be comprehensive, not just basic happy path

### RULE 4: EXACT BEHAVIORAL MATCHING
- Rust implementation must match C++ behavior **EXACTLY**
- Same algorithms, same mathematical precision
- Same edge case handling, same error conditions
- Same performance characteristics (or better)
- **NO** "close enough" implementations

### RULE 5: DATABASE TRACKING MANDATORY
- **EVERY** implementation step must be tracked in SQLite database
- Functions marked as `rust_implemented = 1` **MUST** be complete
- Functions marked as `rust_tested = 1` **MUST** have passing tests
- **NO** marking as complete until requirements are 100% met

### RULE 6: NO SHORTCUTS OR COMPROMISES
- **NO** "temporary workarounds" 
- **NO** "we'll come back to this later"
- **NO** "good enough for now"
- **NO** "this edge case probably doesn't matter"
- If you encounter ANY dependency or complexity issue, **STOP** and resolve dependencies first

## IMPLEMENTATION PROCESS

### Step 1: Dependency Analysis
Before implementing ANY function:
1. Query database for ALL functions called by target function
2. Verify ALL dependencies are marked `rust_implemented = 1` AND `rust_tested = 1` 
3. If ANY dependency is incomplete, **STOP** - implement dependencies first

### Step 2: Implementation
1. Study C++ implementation in detail
2. Understand ALL edge cases and error conditions
3. Implement in Rust with exact behavioral matching
4. Handle ALL the same input validation and error cases

### Step 3: Testing
1. Create comprehensive unit tests
2. Test ALL edge cases, not just happy path
3. Verify exact match with C++ behavior
4. Ensure 100% test pass rate

### Step 4: Database Update
1. Update database: `rust_implemented = 1`
2. Update database: `rust_tested = 1`  
3. **ONLY** after both implementation and testing are complete

## VERIFICATION COMMANDS

Before any implementation session:
```bash
python function_verifier.py
```

This will verify:
- Database completeness (all 790 functions captured)
- Implementation status
- Dependency readiness

## QUALITY GATES

### Gate 1: Dependency Check
**FAIL IMMEDIATELY** if implementing a function with incomplete dependencies

### Gate 2: Implementation Review
**FAIL IMMEDIATELY** if implementation uses any forbidden patterns:
- `todo!()`
- `unimplemented!()`
- `panic!()` for missing functionality
- Stub functions
- Partial implementations

### Gate 3: Testing Verification  
**FAIL IMMEDIATELY** if:
- Tests don't exist
- Tests don't cover edge cases
- Any test fails
- Behavior doesn't exactly match C++

## IMPLEMENTATION DATABASE

Complete analysis database: `clipper2_complete.db`
- **790 functions** total across all files
- **56 classes/structs** 
- **11 enums**
- **857 total items** to implement

## FORBIDDEN PRACTICES

The following are **STRICTLY FORBIDDEN** and will result in immediate rejection:

❌ Writing stub functions  
❌ Using `todo!()` or `unimplemented!()`  
❌ Implementing without dependencies ready  
❌ Skipping comprehensive tests  
❌ "Close enough" implementations  
❌ Marking functions complete prematurely  
❌ Any shortcuts or compromises  

## REQUIRED PRACTICES

The following are **MANDATORY**:

✅ Complete dependency analysis before implementation  
✅ Exact behavioral matching with C++  
✅ Comprehensive unit testing  
✅ Database tracking of all progress  
✅ Zero tolerance for incomplete work  
✅ Production-ready code only  

## REMEMBER

**NO STUBS. NO TODOS. NO EXCEPTIONS.**

Every function must be **perfect** before moving to the next one. This is not negotiable.