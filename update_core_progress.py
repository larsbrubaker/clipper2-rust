#!/usr/bin/env python3
"""
Update core module functions as implemented
"""

import sqlite3

def update_core_functions(db_path: str):
    """Update core functions as implemented and tested"""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    # Mark mathematical functions as implemented
    math_functions = [
        'CrossProduct',
        'DotProduct',
        'MidPoint', 
    ]
    
    for func_name in math_functions:
        cursor.execute('''
            UPDATE functions 
            SET rust_implemented = 1, rust_tested = 1
            WHERE filepath LIKE '%clipper.core.h%' 
            AND name = ?
        ''', (func_name,))
        
        if cursor.rowcount > 0:
            print(f"Marked {cursor.rowcount} {func_name} functions as complete")
    
    # Mark Rect methods as implemented (Width, Height, IsValid, IsEmpty, Scale)
    rect_methods = ['Width', 'Height', 'IsValid', 'IsEmpty', 'Scale']
    for method in rect_methods:
        cursor.execute('''
            UPDATE functions 
            SET rust_implemented = 1, rust_tested = 1
            WHERE filepath LIKE '%clipper.core.h%' 
            AND name = ?
        ''', (method,))
        
        if cursor.rowcount > 0:
            print(f"Marked {cursor.rowcount} {method} methods as complete")
            
    # Mark Point methods as implemented (Negate, Init, SetZ)
    point_methods = ['Negate', 'Init', 'SetZ'] 
    for method in point_methods:
        cursor.execute('''
            UPDATE functions 
            SET rust_implemented = 1, rust_tested = 1
            WHERE filepath LIKE '%clipper.core.h%' 
            AND name = ?
        ''', (method,))
        
        if cursor.rowcount > 0:
            print(f"Marked {cursor.rowcount} {method} methods as complete")
    
    conn.commit()
    
    # Show progress
    cursor.execute('''
        SELECT 
            COUNT(*) as total,
            SUM(CASE WHEN rust_implemented = 1 THEN 1 ELSE 0 END) as implemented,
            SUM(CASE WHEN rust_tested = 1 THEN 1 ELSE 0 END) as tested
        FROM functions
        WHERE filepath LIKE '%clipper.core.h%'
    ''')
    
    total, implemented, tested = cursor.fetchone()
    print(f"\nCore module progress:")
    print(f"  Total functions: {total}")
    print(f"  Implemented: {implemented} ({implemented/total*100:.1f}%)")
    print(f"  Tested: {tested} ({tested/total*100:.1f}%)")
    
    # Show overall project progress
    cursor.execute('''
        SELECT 
            COUNT(*) as total,
            SUM(CASE WHEN rust_implemented = 1 THEN 1 ELSE 0 END) as implemented,
            SUM(CASE WHEN rust_tested = 1 THEN 1 ELSE 0 END) as tested
        FROM functions
    ''')
    
    total_proj, impl_proj, test_proj = cursor.fetchone()
    print(f"\nOverall project progress:")
    print(f"  Total functions: {total_proj}")
    print(f"  Implemented: {impl_proj} ({impl_proj/total_proj*100:.1f}%)")
    print(f"  Tested: {test_proj} ({test_proj/total_proj*100:.1f}%)")
    print(f"  Remaining: {total_proj - impl_proj}")
    
    conn.close()

if __name__ == "__main__":
    db_path = r"C:\Development\clipper2-rust\clipper2_complete.db"
    update_core_functions(db_path)