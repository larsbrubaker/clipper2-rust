#!/usr/bin/env python3
"""
Query functions from database for specific files/modules
"""

import sqlite3
from pathlib import Path

def query_functions_by_file(db_path: str, filename_pattern: str):
    """Query all functions for a specific file"""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    cursor.execute('''
        SELECT name, line_number, signature, return_type, 
               is_constructor, is_destructor, is_operator,
               rust_implemented, rust_tested
        FROM functions 
        WHERE filepath LIKE ?
        ORDER BY line_number
    ''', (f'%{filename_pattern}%',))
    
    functions = cursor.fetchall()
    
    if functions:
        print(f"\nFunctions in {filename_pattern}:")
        print("-" * 60)
        for name, line_num, sig, ret_type, is_ctor, is_dtor, is_op, impl, tested in functions:
            status = ""
            if impl and tested:
                status = "[COMPLETE]"
            elif impl:
                status = "[IMPLEMENTED]"
            elif tested:
                status = "[TESTED]"
            else:
                status = "[TODO]"
                
            type_info = []
            if is_ctor: type_info.append("CTOR")
            if is_dtor: type_info.append("DTOR") 
            if is_op: type_info.append("OPERATOR")
            
            type_str = f" ({', '.join(type_info)})" if type_info else ""
            ret_str = f"{ret_type} " if ret_type else ""
            
            print(f"  {status:12} Line {line_num:3d}: {ret_str}{name}(){type_str}")
            if sig and len(sig) < 100:
                print(f"               Signature: {sig}")
    else:
        print(f"No functions found in {filename_pattern}")
        
    conn.close()
    return len(functions)

def show_file_summary(db_path: str):
    """Show summary of all files and function counts"""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    cursor.execute('''
        SELECT 
            filepath,
            COUNT(*) as func_count,
            SUM(CASE WHEN rust_implemented = 1 THEN 1 ELSE 0 END) as implemented,
            SUM(CASE WHEN rust_tested = 1 THEN 1 ELSE 0 END) as tested
        FROM functions 
        GROUP BY filepath 
        ORDER BY func_count DESC
    ''')
    
    print("\nFILE SUMMARY:")
    print("-" * 80)
    print(f"{'File':<30} {'Functions':<10} {'Implemented':<12} {'Tested':<8} {'%Done':<6}")
    print("-" * 80)
    
    total_funcs = 0
    total_impl = 0
    total_tested = 0
    
    for filepath, func_count, implemented, tested in cursor.fetchall():
        filename = Path(filepath).name
        pct_done = (implemented / func_count * 100) if func_count > 0 else 0
        
        print(f"{filename:<30} {func_count:<10} {implemented:<12} {tested:<8} {pct_done:<6.1f}")
        
        total_funcs += func_count
        total_impl += implemented  
        total_tested += tested
        
    print("-" * 80)
    total_pct = (total_impl / total_funcs * 100) if total_funcs > 0 else 0
    print(f"{'TOTAL':<30} {total_funcs:<10} {total_impl:<12} {total_tested:<8} {total_pct:<6.1f}")
    
    conn.close()

if __name__ == "__main__":
    db_path = r"C:\Development\clipper2-rust\clipper2_complete.db"
    
    # Show overall summary first
    show_file_summary(db_path)
    
    # Query specific file
    print("\n" + "="*60)
    query_functions_by_file(db_path, "clipper.core.h")