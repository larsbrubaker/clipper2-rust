#!/usr/bin/env python3
"""
Simple C++ Code Analysis Tool for Clipper2 Library
Quick analysis to extract basic function and class information
"""

import sqlite3
import re
import os
from pathlib import Path

def create_database(db_path):
    """Create SQLite database with basic schema"""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    # Simple tables for tracking
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY,
            filepath TEXT UNIQUE,
            file_type TEXT
        )
    ''')
    
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS classes (
            id INTEGER PRIMARY KEY,
            name TEXT,
            filepath TEXT,
            line_number INTEGER,
            is_struct BOOLEAN,
            rust_implemented BOOLEAN DEFAULT FALSE
        )
    ''')
    
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS functions (
            id INTEGER PRIMARY KEY,
            name TEXT,
            filepath TEXT,
            line_number INTEGER,
            class_name TEXT,
            return_type TEXT,
            rust_implemented BOOLEAN DEFAULT FALSE,
            rust_tested BOOLEAN DEFAULT FALSE
        )
    ''')
    
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS enums (
            id INTEGER PRIMARY KEY,
            name TEXT,
            filepath TEXT,
            line_number INTEGER,
            rust_implemented BOOLEAN DEFAULT FALSE
        )
    ''')
    
    conn.commit()
    return conn

def analyze_header_file(filepath, conn):
    """Analyze any C++ file for classes, functions, enums"""
    cursor = conn.cursor()
    
    # Determine file type
    path_str = str(filepath).lower()
    if 'test' in path_str:
        file_type = 'test'
    elif 'example' in path_str:
        file_type = 'example'
    elif 'benchmark' in path_str:
        file_type = 'benchmark'
    elif 'utils' in path_str:
        file_type = 'utility'
    elif filepath.suffix == '.h':
        file_type = 'header'
    else:
        file_type = 'source'
    
    # Add file to database
    cursor.execute('INSERT OR IGNORE INTO files (filepath, file_type) VALUES (?, ?)',
                   (str(filepath), file_type))
    
    try:
        with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()
    except:
        return
        
    print(f"Analyzing: {filepath.name} ({file_type})")
    
    # Extract classes/structs
    class_pattern = r'^\s*(class|struct)\s+(?:CLIPPER2_DLL\s+)?(\w+)'
    for match in re.finditer(class_pattern, content, re.MULTILINE):
        is_struct = match.group(1) == 'struct'
        name = match.group(2)
        line_number = content[:match.start()].count('\n') + 1
        
        cursor.execute('''
            INSERT OR IGNORE INTO classes 
            (name, filepath, line_number, is_struct) 
            VALUES (?, ?, ?, ?)
        ''', (name, str(filepath), line_number, is_struct))
        
    # Extract enums
    enum_pattern = r'^\s*enum\s+(?:class\s+)?(\w+)'
    for match in re.finditer(enum_pattern, content, re.MULTILINE):
        name = match.group(1)
        line_number = content[:match.start()].count('\n') + 1
        
        cursor.execute('''
            INSERT OR IGNORE INTO enums (name, filepath, line_number)
            VALUES (?, ?, ?)
        ''', (name, str(filepath), line_number))
        
    # Extract functions (simplified)
    # Look for function declarations/definitions
    func_pattern = r'^\s*(?:(?:inline|static|virtual|explicit|const|constexpr|template<[^>]*>)\s+)*([a-zA-Z_:]\w*(?:\s*[*&])*)\s+([a-zA-Z_]\w*)\s*\([^)]*\)\s*(?:const)?\s*(?:override)?\s*[;{]'
    
    for match in re.finditer(func_pattern, content, re.MULTILINE):
        return_type = match.group(1).strip()
        func_name = match.group(2)
        
        # Skip obvious keywords and constructors that match class names
        if return_type in ['if', 'while', 'for', 'switch', 'return', 'namespace']:
            continue
        if len(func_name) < 2:
            continue
            
        line_number = content[:match.start()].count('\n') + 1
        
        cursor.execute('''
            INSERT OR IGNORE INTO functions 
            (name, filepath, line_number, return_type) 
            VALUES (?, ?, ?, ?)
        ''', (func_name, str(filepath), line_number, return_type))
        
    conn.commit()

def main():
    cpp_root = Path(r"C:\Development\clipper2-rust\CPP")
    db_path = r"C:\Development\clipper2-rust\clipper2_simple.db"
    
    # Remove existing database
    if os.path.exists(db_path):
        os.remove(db_path)
        
    conn = create_database(db_path)
    
    # Analyze ALL files - headers and source files
    all_files = []
    
    # Core library headers
    clipper2lib_dir = cpp_root / "Clipper2Lib" / "include" / "clipper2"
    if clipper2lib_dir.exists():
        all_files.extend(clipper2lib_dir.glob("*.h"))
        
    # Core library source files
    clipper2lib_src_dir = cpp_root / "Clipper2Lib" / "src"
    if clipper2lib_src_dir.exists():
        all_files.extend(clipper2lib_src_dir.glob("*.cpp"))
        
    # Utils headers and source
    utils_dir = cpp_root / "Utils"
    if utils_dir.exists():
        all_files.extend(utils_dir.glob("*.h"))
        all_files.extend(utils_dir.glob("*.cpp"))
        
    # Examples
    examples_dir = cpp_root / "Examples"
    if examples_dir.exists():
        all_files.extend(examples_dir.rglob("*.cpp"))
        all_files.extend(examples_dir.rglob("*.h"))
        
    # Tests
    tests_dir = cpp_root / "Tests"
    if tests_dir.exists():
        all_files.extend(tests_dir.glob("*.cpp"))
        all_files.extend(tests_dir.glob("*.h"))
        
    # Benchmarks
    benchmark_dir = cpp_root / "BenchMark"
    if benchmark_dir.exists():
        all_files.extend(benchmark_dir.glob("*.cpp"))
        all_files.extend(benchmark_dir.glob("*.h"))
    
    print(f"Found {len(all_files)} files to analyze")
    
    for file_path in all_files:
        analyze_header_file(file_path, conn)
        
    # Print summary
    cursor = conn.cursor()
    
    print("\n=== ANALYSIS SUMMARY ===")
    
    cursor.execute('SELECT COUNT(*) FROM classes')
    print(f"Classes/Structs: {cursor.fetchone()[0]}")
    
    cursor.execute('SELECT COUNT(*) FROM functions')  
    print(f"Functions: {cursor.fetchone()[0]}")
    
    cursor.execute('SELECT COUNT(*) FROM enums')
    print(f"Enums: {cursor.fetchone()[0]}")
    
    print("\n=== CLASSES ===")
    cursor.execute('SELECT name, filepath, is_struct FROM classes ORDER BY name')
    for name, filepath, is_struct in cursor.fetchall():
        type_str = "struct" if is_struct else "class"
        print(f"{type_str} {name} - {Path(filepath).name}")
        
    print("\n=== KEY FUNCTIONS (first 20) ===")
    cursor.execute('SELECT name, return_type, filepath FROM functions ORDER BY name LIMIT 20')
    for name, return_type, filepath in cursor.fetchall():
        print(f"{return_type} {name}() - {Path(filepath).name}")
    
    conn.close()
    print(f"\nDatabase saved to: {db_path}")

if __name__ == "__main__":
    main()