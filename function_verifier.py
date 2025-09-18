#!/usr/bin/env python3
"""
Function Verification Tool for Clipper2 Library Port
Ensures EVERY function from C++ is recorded and will be implemented
NO EXCEPTIONS - NO STUBS - NO TODOS ALLOWED
"""

import sqlite3
import re
from pathlib import Path
from typing import Dict, List, Set, Tuple

class FunctionVerifier:
    def __init__(self, db_path: str, cpp_root: str):
        self.db_path = db_path
        self.cpp_root = Path(cpp_root)
        self.conn = sqlite3.connect(db_path)
        
    def extract_all_functions_from_source(self) -> Dict[str, List[Tuple[str, int, str]]]:
        """Extract ALL functions directly from C++ source files"""
        all_functions = {}
        
        for cpp_file in self.cpp_root.rglob("*.cpp"):
            functions = self._extract_functions_from_file(cpp_file)
            if functions:
                all_functions[str(cpp_file)] = functions
                
        for h_file in self.cpp_root.rglob("*.h"):
            functions = self._extract_functions_from_file(h_file)
            if functions:
                all_functions[str(h_file)] = functions
                
        return all_functions
        
    def _extract_functions_from_file(self, filepath: Path) -> List[Tuple[str, int, str]]:
        """Extract function signatures from a single file"""
        try:
            with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
        except:
            return []
            
        functions = []
        
        # Enhanced regex patterns for different function types
        patterns = [
            # Regular functions with return type
            r'^\s*(?:(?:inline|static|virtual|explicit|const|constexpr|template<[^>]*>)\s+)*([a-zA-Z_:]\w*(?:\s*[*&])*)\s+([a-zA-Z_]\w*)\s*\([^)]*\)\s*(?:const)?\s*(?:override)?\s*[{;]',
            # Constructor/Destructor patterns
            r'^\s*(?:explicit\s+)?([a-zA-Z_]\w*)\s*\([^)]*\)\s*[{;:]',
            r'^\s*~([a-zA-Z_]\w*)\s*\([^)]*\)\s*[{;]',
            # Operator overloads
            r'^\s*(?:[a-zA-Z_:]\w*(?:\s*[*&])*)\s+(operator\s*[^\s(]+)\s*\([^)]*\)\s*(?:const)?\s*[{;]',
        ]
        
        lines = content.split('\n')
        for line_num, line in enumerate(lines, 1):
            original_line = line
            line = line.strip()
            
            # Skip preprocessor, comments, and obvious non-functions
            if (line.startswith('#') or line.startswith('//') or 
                line.startswith('/*') or not line or
                line.startswith('namespace') or line.startswith('using') or
                line.startswith('typedef')):
                continue
                
            for pattern in patterns:
                matches = re.finditer(pattern, original_line, re.MULTILINE)
                for match in matches:
                    if len(match.groups()) >= 2:
                        func_name = match.group(2)
                    else:
                        func_name = match.group(1)
                        
                    # Filter out obvious non-functions
                    if (func_name in ['if', 'while', 'for', 'switch', 'return', 'namespace', 'class', 'struct'] or
                        len(func_name) < 2 or func_name.isdigit()):
                        continue
                        
                    full_match = match.group(0)
                    functions.append((func_name, line_num, full_match.strip()))
                    
        return functions
        
    def verify_database_completeness(self):
        """Verify that our database contains ALL functions from source"""
        print("VERIFYING DATABASE COMPLETENESS")
        print("=" * 60)
        
        # Get functions from database
        cursor = self.conn.cursor()
        cursor.execute('SELECT name, filepath, line_number FROM functions ORDER BY filepath, line_number')
        db_functions = {}
        for name, filepath, line_num in cursor.fetchall():
            if filepath not in db_functions:
                db_functions[filepath] = []
            db_functions[filepath].append((name, line_num))
            
        # Get functions from source
        source_functions = self.extract_all_functions_from_source()
        
        # Compare
        missing_functions = []
        extra_functions = []
        
        for filepath, functions in source_functions.items():
            source_names = {name for name, _, _ in functions}
            db_names = {name for name, _ in db_functions.get(filepath, [])}
            
            missing = source_names - db_names
            extra = db_names - source_names
            
            if missing:
                for func_name in missing:
                    # Find line number
                    line_num = next((line for name, line, _ in functions if name == func_name), 0)
                    missing_functions.append((func_name, filepath, line_num))
                    
            if extra:
                for func_name in extra:
                    line_num = next((line for name, line in db_functions[filepath] if name == func_name), 0)
                    extra_functions.append((func_name, filepath, line_num))
                    
        # Report results
        print(f"Database contains: {sum(len(funcs) for funcs in db_functions.values())} functions")
        print(f"Source contains: {sum(len(funcs) for funcs in source_functions.values())} functions")
        
        if missing_functions:
            print(f"\nMISSING FUNCTIONS IN DATABASE: {len(missing_functions)}")
            print("-" * 40)
            for func_name, filepath, line_num in sorted(missing_functions):
                print(f"  {func_name} - {Path(filepath).name}:{line_num}")
                
        if extra_functions:
            print(f"\nEXTRA FUNCTIONS IN DATABASE: {len(extra_functions)}")
            print("-" * 40)
            for func_name, filepath, line_num in sorted(extra_functions):
                print(f"  {func_name} - {Path(filepath).name}:{line_num}")
                
        if not missing_functions and not extra_functions:
            print("\nDATABASE IS COMPLETE - All functions recorded!")
        else:
            print(f"\nX DATABASE IS INCOMPLETE - {len(missing_functions)} missing, {len(extra_functions)} extra")
            
        return len(missing_functions) == 0 and len(extra_functions) == 0
        
    def generate_function_report(self):
        """Generate comprehensive function implementation report"""
        cursor = self.conn.cursor()
        
        # Get comprehensive stats
        cursor.execute('''
            SELECT 
                f.file_type,
                COUNT(fn.id) as function_count,
                SUM(CASE WHEN fn.rust_implemented = 1 THEN 1 ELSE 0 END) as implemented_count,
                SUM(CASE WHEN fn.rust_tested = 1 THEN 1 ELSE 0 END) as tested_count
            FROM files f 
            LEFT JOIN functions fn ON f.filepath = fn.filepath 
            GROUP BY f.file_type 
            ORDER BY function_count DESC
        ''')
        
        print("\nFUNCTION IMPLEMENTATION REPORT")
        print("=" * 60)
        
        total_functions = 0
        total_implemented = 0
        total_tested = 0
        
        for file_type, func_count, impl_count, test_count in cursor.fetchall():
            if func_count > 0:
                impl_pct = (impl_count / func_count) * 100
                test_pct = (test_count / func_count) * 100
                print(f"{file_type:12} | {func_count:4d} functions | {impl_count:4d} impl ({impl_pct:5.1f}%) | {test_count:4d} tested ({test_pct:5.1f}%)")
                
                total_functions += func_count
                total_implemented += impl_count
                total_tested += test_count
                
        print("-" * 60)
        if total_functions > 0:
            total_impl_pct = (total_implemented / total_functions) * 100
            total_test_pct = (total_tested / total_functions) * 100
            print(f"{'TOTAL':12} | {total_functions:4d} functions | {total_implemented:4d} impl ({total_impl_pct:5.1f}%) | {total_tested:4d} tested ({total_test_pct:5.1f}%)")
            
    def create_implementation_checklist(self):
        """Create detailed implementation checklist"""
        cursor = self.conn.cursor()
        
        # Get all functions grouped by file
        cursor.execute('''
            SELECT 
                f.filepath, f.file_type,
                fn.name, fn.line_number, fn.return_type,
                fn.rust_implemented, fn.rust_tested
            FROM functions fn
            JOIN files f ON fn.filepath = f.filepath
            ORDER BY 
                CASE f.file_type 
                    WHEN 'header' THEN 1
                    WHEN 'source' THEN 2  
                    WHEN 'utility' THEN 3
                    WHEN 'test' THEN 4
                    WHEN 'example' THEN 5
                    WHEN 'benchmark' THEN 6
                    ELSE 7
                END,
                f.filepath, fn.line_number
        ''')
        
        checklist_content = """# CLIPPER2 RUST PORT - COMPLETE IMPLEMENTATION CHECKLIST

## STRICT IMPLEMENTATION RULES

### RULE 1: ZERO TOLERANCE POLICY
- NO function stubs allowed (no `todo!()`, `unimplemented!()`, or `panic!()`)
- NO partial implementations 
- NO placeholder functions
- EVERY function must be complete and working before marking as implemented

### RULE 2: DEPENDENCY ENFORCEMENT  
- Functions CANNOT be implemented if their dependencies are incomplete
- Must wait for ALL dependency functions to be fully implemented AND tested
- Implementation order MUST follow dependency chain strictly

### RULE 3: TESTING MANDATORY
- EVERY function must have comprehensive unit tests before marking complete
- Tests must cover all edge cases and match C++ behavior exactly
- No function is "complete" until tests pass 100%

### RULE 4: EXACT BEHAVIOR MATCHING
- Rust implementation must match C++ behavior EXACTLY
- Same algorithms, same edge cases, same error conditions
- Performance characteristics should be equivalent or better

---

## IMPLEMENTATION CHECKLIST

"""
        
        current_file = None
        current_type = None
        
        for filepath, file_type, func_name, line_num, return_type, rust_impl, rust_tested in cursor.fetchall():
            if file_type != current_type:
                checklist_content += f"\n## {file_type.upper()} FILES\n\n"
                current_type = file_type
                current_file = None
                
            if filepath != current_file:
                checklist_content += f"\n### {Path(filepath).name}\n"
                current_file = filepath
                
            # Status indicators
            impl_status = "DONE" if rust_impl else "TODO"
            test_status = "TESTED" if rust_tested else "PENDING"
            
            return_str = return_type if return_type else "void"
            checklist_content += f"- {impl_status} {test_status} `{return_str} {func_name}()` (line {line_num})\n"
            
        # Add summary
        cursor.execute('SELECT COUNT(*) FROM functions')
        total_functions = cursor.fetchone()[0]
        
        cursor.execute('SELECT COUNT(*) FROM functions WHERE rust_implemented = 1')
        implemented = cursor.fetchone()[0]
        
        cursor.execute('SELECT COUNT(*) FROM functions WHERE rust_tested = 1')
        tested = cursor.fetchone()[0]
        
        checklist_content += f"""
---

## IMPLEMENTATION PROGRESS

- **Total Functions**: {total_functions}
- **Implemented**: {implemented} ({implemented/total_functions*100:.1f}%)
- **Tested**: {tested} ({tested/total_functions*100:.1f}%)
- **Remaining**: {total_functions - implemented} functions

## LEGEND

- TODO PENDING = Not implemented
- DONE TESTED = Fully implemented and tested
- First status = Implementation status  
- Second status = Test status

**REMEMBER: NO EXCEPTIONS TO THE RULES!**
"""
        
        with open("C:\\Development\\clipper2-rust\\IMPLEMENTATION_CHECKLIST.md", 'w') as f:
            f.write(checklist_content)
            
        print(f"\nImplementation checklist created: IMPLEMENTATION_CHECKLIST.md")
        print(f"Total functions to implement: {total_functions}")
        
    def close(self):
        self.conn.close()

def main():
    db_path = r"C:\Development\clipper2-rust\clipper2_complete.db"
    cpp_root = r"C:\Development\clipper2-rust\CPP"
    
    verifier = FunctionVerifier(db_path, cpp_root)
    
    # Verify database completeness
    is_complete = verifier.verify_database_completeness()
    
    # Generate reports
    verifier.generate_function_report()
    verifier.create_implementation_checklist()
    
    if not is_complete:
        print("\nWARNING: Database is not complete!")
        print("   Please update the analyzer to capture all functions")
        print("   NO implementation should begin until ALL functions are recorded")
    else:
        print("\nREADY to begin STRICT implementation process")
        print("   Remember: NO STUBS, NO TODOS, NO EXCEPTIONS!")
        
    verifier.close()

if __name__ == "__main__":
    main()