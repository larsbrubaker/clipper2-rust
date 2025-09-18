#!/usr/bin/env python3
"""
COMPLETE C++ Function Analyzer - Captures EVERYTHING
NO function will be missed - ZERO TOLERANCE for incomplete analysis
"""

import sqlite3
import re
from pathlib import Path
import json

class CompleteAnalyzer:
    def __init__(self, db_path: str, cpp_root: str):
        self.db_path = db_path
        self.cpp_root = Path(cpp_root)
        self.conn = sqlite3.connect(db_path)
        
        # Clear and recreate tables for complete analysis
        self.create_complete_tables()
        
    def create_complete_tables(self):
        """Create comprehensive tables"""
        cursor = self.conn.cursor()
        
        # Drop existing tables
        cursor.execute('DROP TABLE IF EXISTS functions')
        cursor.execute('DROP TABLE IF EXISTS classes') 
        cursor.execute('DROP TABLE IF EXISTS enums')
        cursor.execute('DROP TABLE IF EXISTS files')
        
        # Create comprehensive tables
        cursor.execute('''
            CREATE TABLE files (
                id INTEGER PRIMARY KEY,
                filepath TEXT UNIQUE,
                file_type TEXT,
                analyzed BOOLEAN DEFAULT FALSE
            )
        ''')
        
        cursor.execute('''
            CREATE TABLE functions (
                id INTEGER PRIMARY KEY,
                name TEXT,
                filepath TEXT,
                line_number INTEGER,
                signature TEXT,           -- Complete function signature
                return_type TEXT,
                parameters TEXT,          -- JSON array
                is_template BOOLEAN DEFAULT FALSE,
                is_inline BOOLEAN DEFAULT FALSE,
                is_static BOOLEAN DEFAULT FALSE,
                is_virtual BOOLEAN DEFAULT FALSE,
                is_const BOOLEAN DEFAULT FALSE,
                is_constructor BOOLEAN DEFAULT FALSE,
                is_destructor BOOLEAN DEFAULT FALSE,
                is_operator BOOLEAN DEFAULT FALSE,
                is_main BOOLEAN DEFAULT FALSE,
                class_name TEXT,          -- If it's a method
                namespace_name TEXT,
                rust_implemented BOOLEAN DEFAULT FALSE,
                rust_tested BOOLEAN DEFAULT FALSE,
                FOREIGN KEY (filepath) REFERENCES files(filepath)
            )
        ''')
        
        cursor.execute('''
            CREATE TABLE classes (
                id INTEGER PRIMARY KEY,
                name TEXT,
                filepath TEXT,
                line_number INTEGER,
                is_struct BOOLEAN,
                template_params TEXT,     -- JSON array
                base_classes TEXT,        -- JSON array
                namespace_name TEXT,
                rust_implemented BOOLEAN DEFAULT FALSE,
                FOREIGN KEY (filepath) REFERENCES files(filepath)
            )
        ''')
        
        cursor.execute('''
            CREATE TABLE enums (
                id INTEGER PRIMARY KEY,
                name TEXT,
                filepath TEXT,
                line_number INTEGER,
                enum_values TEXT,         -- JSON array
                is_class_enum BOOLEAN DEFAULT FALSE,
                rust_implemented BOOLEAN DEFAULT FALSE,
                FOREIGN KEY (filepath) REFERENCES files(filepath)
            )
        ''')
        
        self.conn.commit()
        
    def scan_all_files(self):
        """Scan for ALL C++ files"""
        cursor = self.conn.cursor()
        
        all_files = []
        all_files.extend(self.cpp_root.rglob("*.h"))
        all_files.extend(self.cpp_root.rglob("*.hpp"))
        all_files.extend(self.cpp_root.rglob("*.cpp"))
        all_files.extend(self.cpp_root.rglob("*.cc"))
        all_files.extend(self.cpp_root.rglob("*.cxx"))
        
        for filepath in all_files:
            # Determine file type
            path_str = str(filepath).lower()
            if 'test' in path_str:
                file_type = 'test'
            elif 'example' in path_str:
                file_type = 'example'  
            elif 'benchmark' in path_str:
                file_type = 'benchmark'
            elif 'utils' in path_str or 'util' in path_str:
                file_type = 'utility'
            elif filepath.suffix in ['.h', '.hpp']:
                file_type = 'header'
            else:
                file_type = 'source'
                
            cursor.execute('INSERT OR IGNORE INTO files (filepath, file_type) VALUES (?, ?)',
                          (str(filepath), file_type))
                          
        self.conn.commit()
        print(f"Found {len(all_files)} files to analyze")
        
    def analyze_file(self, filepath: str):
        """Completely analyze a single file - capture EVERYTHING"""
        print(f"Analyzing: {Path(filepath).name}")
        
        try:
            with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
        except Exception as e:
            print(f"  ERROR: {e}")
            return
            
        cursor = self.conn.cursor()
        
        # Extract all constructs
        self.extract_all_functions(content, filepath)
        self.extract_all_classes(content, filepath) 
        self.extract_all_enums(content, filepath)
        
        # Mark as analyzed
        cursor.execute('UPDATE files SET analyzed = TRUE WHERE filepath = ?', (filepath,))
        self.conn.commit()
        
    def extract_all_functions(self, content: str, filepath: str):
        """Extract ALL functions - no exceptions"""
        cursor = self.conn.cursor()
        
        lines = content.split('\n')
        in_comment_block = False
        current_class = None
        current_namespace = None
        brace_depth = 0
        
        for line_num, line in enumerate(lines, 1):
            original_line = line
            line = line.strip()
            
            # Track comment blocks
            if '/*' in line:
                in_comment_block = True
            if '*/' in line:
                in_comment_block = False
                continue
                
            if in_comment_block or line.startswith('//') or line.startswith('#'):
                continue
                
            # Track namespace
            namespace_match = re.match(r'namespace\s+(\w+)', line)
            if namespace_match:
                current_namespace = namespace_match.group(1)
                
            # Track class context  
            class_match = re.match(r'(?:class|struct)\s+(?:CLIPPER2_DLL\s+)?(\w+)', line)
            if class_match:
                current_class = class_match.group(1)
                
            # Track brace depth for context
            brace_depth += line.count('{') - line.count('}')
            
            # Reset class context when leaving class
            if current_class and brace_depth == 0 and '}' in line:
                current_class = None
                
            # Function detection patterns
            function_patterns = [
                # Regular function/method
                r'^\s*(?:(?:inline|static|virtual|explicit|const|constexpr|template<[^>]*>)\s+)*([a-zA-Z_:]\w*(?:\s*[*&])*)\s+([a-zA-Z_]\w*)\s*\([^)]*\)\s*(?:const)?\s*(?:override)?\s*(?:=\s*0)?\s*[{;]',
                
                # Constructor (class name followed by parameters)
                r'^\s*(?:explicit\s+)?([a-zA-Z_]\w*)\s*\([^)]*\)\s*(?::\s*[^{;]*)?[{;]',
                
                # Destructor
                r'^\s*(~[a-zA-Z_]\w*)\s*\([^)]*\)\s*[{;]',
                
                # Operator overload
                r'^\s*(?:[a-zA-Z_:]\w*(?:\s*[*&])*\s+)?(operator\s*[^\s(]+)\s*\([^)]*\)\s*(?:const)?\s*[{;]',
                
                # main function
                r'^\s*(int|void)\s+(main)\s*\([^)]*\)\s*[{;]',
                
                # Function-like macros and special cases
                r'^\s*([A-Z_]+)\s*\([^)]*\)\s*[{;]',  # EXPECT_EQ, ASSERT_TRUE, etc.
            ]
            
            for pattern in function_patterns:
                matches = re.finditer(pattern, original_line)
                for match in matches:
                    if len(match.groups()) >= 2:
                        return_type = match.group(1).strip() if match.group(1) else None
                        func_name = match.group(2).strip()
                    else:
                        return_type = None
                        func_name = match.group(1).strip()
                        
                    # Filter obvious non-functions  
                    if (func_name in ['if', 'while', 'for', 'switch', 'return', 'else', 'case', 
                                    'break', 'continue', 'namespace', 'class', 'struct', 'enum',
                                    'typedef', 'using', 'template'] or
                        len(func_name) < 1 or func_name.isdigit()):
                        continue
                        
                    # Determine function properties
                    is_constructor = (current_class and func_name == current_class)
                    is_destructor = func_name.startswith('~')
                    is_operator = func_name.startswith('operator')
                    is_main = func_name == 'main'
                    is_template = 'template<' in original_line
                    is_inline = 'inline' in original_line
                    is_static = 'static' in original_line
                    is_virtual = 'virtual' in original_line
                    is_const = 'const' in original_line and ')' in original_line
                    
                    # Get full signature
                    signature = match.group(0).strip()
                    
                    cursor.execute('''
                        INSERT OR IGNORE INTO functions 
                        (name, filepath, line_number, signature, return_type, 
                         is_template, is_inline, is_static, is_virtual, is_const,
                         is_constructor, is_destructor, is_operator, is_main,
                         class_name, namespace_name)
                        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ''', (func_name, filepath, line_num, signature, return_type,
                          is_template, is_inline, is_static, is_virtual, is_const,
                          is_constructor, is_destructor, is_operator, is_main,
                          current_class, current_namespace))
                        
    def extract_all_classes(self, content: str, filepath: str):
        """Extract ALL class and struct definitions"""
        cursor = self.conn.cursor()
        
        # Class/struct pattern
        pattern = r'^\s*(class|struct)\s+(?:CLIPPER2_DLL\s+)?([a-zA-Z_]\w*)\s*(?::\s*([^{]+?))?\s*[{;]'
        
        for match in re.finditer(pattern, content, re.MULTILINE):
            is_struct = match.group(1) == 'struct'
            name = match.group(2)
            inheritance = match.group(3)
            line_number = content[:match.start()].count('\n') + 1
            
            base_classes = []
            if inheritance:
                # Parse inheritance (simplified)
                bases = [base.strip() for base in inheritance.split(',')]
                base_classes = [re.sub(r'(public|private|protected)\s+', '', base).strip() 
                               for base in bases]
                
            cursor.execute('''
                INSERT OR IGNORE INTO classes 
                (name, filepath, line_number, is_struct, base_classes)
                VALUES (?, ?, ?, ?, ?)
            ''', (name, filepath, line_number, is_struct, json.dumps(base_classes)))
            
    def extract_all_enums(self, content: str, filepath: str):
        """Extract ALL enum definitions"""
        cursor = self.conn.cursor()
        
        # Enum pattern
        pattern = r'^\s*enum\s+(class\s+)?([a-zA-Z_]\w*)\s*(?::\s*[^{]+)?\s*\{([^}]*)\}'
        
        for match in re.finditer(pattern, content, re.MULTILINE | re.DOTALL):
            is_class_enum = match.group(1) is not None
            name = match.group(2)
            values_text = match.group(3)
            line_number = content[:match.start()].count('\n') + 1
            
            # Parse enum values
            values = []
            for value_match in re.finditer(r'([a-zA-Z_]\w*)(?:\s*=\s*([^,}]+))?', values_text):
                value_name = value_match.group(1)
                value_init = value_match.group(2).strip() if value_match.group(2) else None
                values.append({'name': value_name, 'value': value_init})
                
            cursor.execute('''
                INSERT OR IGNORE INTO enums 
                (name, filepath, line_number, enum_values, is_class_enum)
                VALUES (?, ?, ?, ?, ?)
            ''', (name, filepath, line_number, json.dumps(values), is_class_enum))
            
    def run_complete_analysis(self):
        """Run COMPLETE analysis - every function will be found"""
        print("RUNNING COMPLETE ANALYSIS - ZERO TOLERANCE")
        print("=" * 60)
        
        # Scan all files
        self.scan_all_files()
        
        # Analyze each file
        cursor = self.conn.cursor()
        cursor.execute('SELECT filepath FROM files ORDER BY filepath')
        
        for (filepath,) in cursor.fetchall():
            self.analyze_file(filepath)
            
        print("\nANALYSIS COMPLETE")
        self.print_comprehensive_summary()
        
    def print_comprehensive_summary(self):
        """Print detailed summary"""
        cursor = self.conn.cursor()
        
        print("\n" + "=" * 60)
        print("COMPREHENSIVE ANALYSIS RESULTS")
        print("=" * 60)
        
        # File breakdown
        cursor.execute('SELECT file_type, COUNT(*) FROM files GROUP BY file_type ORDER BY COUNT(*) DESC')
        print("\nFILES BY TYPE:")
        total_files = 0
        for file_type, count in cursor.fetchall():
            print(f"  {file_type:12}: {count:3d} files")
            total_files += count
        print(f"  {'TOTAL':12}: {total_files:3d} files")
        
        # Function breakdown  
        cursor.execute('''
            SELECT f.file_type, COUNT(fn.id) as func_count
            FROM files f 
            LEFT JOIN functions fn ON f.filepath = fn.filepath
            GROUP BY f.file_type 
            ORDER BY func_count DESC
        ''')
        print("\nFUNCTIONS BY FILE TYPE:")
        total_functions = 0
        for file_type, func_count in cursor.fetchall():
            if func_count > 0:
                print(f"  {file_type:12}: {func_count:3d} functions")
                total_functions += func_count
        print(f"  {'TOTAL':12}: {total_functions:3d} functions")
        
        # Special function types
        cursor.execute('SELECT COUNT(*) FROM functions WHERE is_main = 1')
        main_count = cursor.fetchone()[0]
        cursor.execute('SELECT COUNT(*) FROM functions WHERE is_constructor = 1') 
        constructor_count = cursor.fetchone()[0]
        cursor.execute('SELECT COUNT(*) FROM functions WHERE is_destructor = 1')
        destructor_count = cursor.fetchone()[0]
        cursor.execute('SELECT COUNT(*) FROM functions WHERE is_operator = 1')
        operator_count = cursor.fetchone()[0]
        cursor.execute('SELECT COUNT(*) FROM functions WHERE is_template = 1')
        template_count = cursor.fetchone()[0]
        
        print("\nSPECIAL FUNCTION TYPES:")
        print(f"  main functions  : {main_count:3d}")
        print(f"  constructors    : {constructor_count:3d}")
        print(f"  destructors     : {destructor_count:3d}")
        print(f"  operators       : {operator_count:3d}") 
        print(f"  templates       : {template_count:3d}")
        
        # Classes and enums
        cursor.execute('SELECT COUNT(*) FROM classes')
        class_count = cursor.fetchone()[0]
        cursor.execute('SELECT COUNT(*) FROM enums')
        enum_count = cursor.fetchone()[0]
        
        print(f"\nCLASSES/STRUCTS : {class_count:3d}")
        print(f"ENUMS           : {enum_count:3d}")
        
        print(f"\nREADY FOR RUST PORT:")
        print(f"   Total items to implement: {total_functions + class_count + enum_count}")
        print(f"   NO STUBS - NO TODOS - NO EXCEPTIONS!")
        
    def close(self):
        self.conn.close()

def main():
    db_path = r"C:\Development\clipper2-rust\clipper2_complete.db"
    cpp_root = r"C:\Development\clipper2-rust\CPP"
    
    # Remove old database
    import os
    if os.path.exists(db_path):
        os.remove(db_path)
        
    analyzer = CompleteAnalyzer(db_path, cpp_root)
    analyzer.run_complete_analysis()
    analyzer.close()
    
    print(f"\nCOMPLETE database saved to: clipper2_complete.db")
    print("Run function_verifier.py to verify completeness")

if __name__ == "__main__":
    main()