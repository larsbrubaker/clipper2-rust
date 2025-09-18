# 🚀 CLIPPER2 RUST PORT - QUICK START

*For spinning up new chat sessions quickly*

## ⚡ IMMEDIATE COMMANDS

```bash
# 1. Check current status (ALWAYS RUN FIRST)
python function_verifier.py

# 2. Test what's implemented
cargo test --lib

# 3. Check database stats
sqlite3 clipper2_complete.db "SELECT COUNT(*) as total, SUM(rust_implemented) as done FROM functions;"
```

## 📊 CURRENT STATUS

- **📁 Total Functions**: 805
- **✅ Implemented**: 53 (6.6%)
- **🧪 Tested**: 53 (6.6%)
- **📋 Remaining**: 752

## 🏗️ WHAT'S COMPLETE

- **✅ core.rs**: All basic types, math, utilities (45+ functions)
- **✅ version.rs**: Version constants
- **✅ Database**: Complete analysis of C++ codebase
- **❌ engine.rs**: Main clipping algorithms (NOT STARTED)
- **❌ offset.rs**: Path offsetting (NOT STARTED)
- **❌ rectclip.rs**: Rectangle clipping (NOT STARTED)

## 🎯 WHAT TO IMPLEMENT NEXT

1. **Rectangle Clipping** (`rectclip.rs`) - Simpler than engine
2. **Engine Data Structures** - Basic types from engine.h
3. **Missing Core Utilities** - Check if any core functions missing

## ⚠️ CRITICAL RULES

❌ **NEVER USE**: `todo!()`, `unimplemented!()`, `panic!()` for missing features
❌ **NO STUBS**: Every function must be 100% complete
❌ **CHECK DEPENDENCIES**: Don't implement if dependencies aren't ready
✅ **FULL TESTS**: Every function needs comprehensive test coverage
✅ **MATCH C++**: Behavior must be identical to C++ version

## 🔍 FINDING NEXT FUNCTIONS

```sql
-- Find ready functions
SELECT name, filepath, line_number FROM functions 
WHERE rust_implemented = 0 
    AND filepath LIKE '%header%'
    AND class_name IS NULL 
LIMIT 10;
```

## 📁 KEY FILES

- **CLAUDE.md**: Implementation rules (READ THIS!)
- **PROJECT_STATUS.md**: Detailed status
- **clipper2_complete.db**: Function database
- **function_verifier.py**: Status checker
- **src/core.rs**: Complete implementation
- **src/lib.rs**: Main library file

**🎯 Ready to continue! Run the commands above to get current status.**