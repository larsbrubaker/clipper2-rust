#!/usr/bin/env python3
"""
Check rectclip dependencies in database
"""

import sqlite3
import os

def check_rectclip_dependencies():
    db_path = "clipper2_complete.db"
    
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    # Look for specific functions that rectclip depends on
    functions_to_check = [
        'GetSegmentIntersectPt',
        'PointInPolygon', 
        'IsCollinear',
        'CrossProduct',
        'GetBounds'
    ]
    
    print('Checking rectclip dependencies:')
    print('-' * 60)
    
    for func_name in functions_to_check:
        cursor.execute('''
            SELECT name, filepath, rust_implemented, rust_tested, signature
            FROM functions 
            WHERE name LIKE ?
            ORDER BY filepath
        ''', (f'%{func_name}%',))
        
        results = cursor.fetchall()
        
        if results:
            print(f'\n{func_name}:')
            for name, filepath, impl, tested, sig in results:
                status = 'COMPLETE' if impl and tested else 'IMPLEMENTED' if impl else 'TODO'
                filename = os.path.basename(filepath)
                print(f'  {status:10} - {filename}: {name}')
                if sig and len(sig) < 100:
                    print(f'               {sig}')
        else:
            print(f'\n{func_name}: NOT FOUND')
    
    conn.close()

if __name__ == "__main__":
    check_rectclip_dependencies()