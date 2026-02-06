#!/usr/bin/env python3
"""
Search for all functions that might be rectclip dependencies
"""

import sqlite3
import os

def search_all_dependencies():
    db_path = "clipper2_complete.db"
    
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    # Search patterns for functions that might be rectclip dependencies
    search_patterns = [
        '%PointInPolygon%',
        '%IsCollinear%', 
        '%Collinear%',
        '%GetBounds%',
        '%Bounds%',
        '%GetSegmentIntersect%',
        '%SegmentIntersect%'
    ]
    
    print('Searching for potential rectclip dependencies:')
    print('-' * 80)
    
    for pattern in search_patterns:
        cursor.execute('''
            SELECT name, filepath, rust_implemented, rust_tested, signature
            FROM functions 
            WHERE name LIKE ?
            ORDER BY filepath, name
        ''', (pattern,))
        
        results = cursor.fetchall()
        
        if results:
            print(f'\nMatches for {pattern}:')
            for name, filepath, impl, tested, sig in results:
                status = 'COMPLETE' if impl and tested else 'IMPLEMENTED' if impl else 'TODO'
                filename = os.path.basename(filepath)
                print(f'  {status:10} - {filename:25} {name}')
    
    # Also look at all core functions to see what's already available
    print('\n' + '='*80)
    print('ALL CORE.H FUNCTIONS:')
    print('='*80)
    
    cursor.execute('''
        SELECT name, rust_implemented, rust_tested, signature
        FROM functions 
        WHERE filepath LIKE '%clipper.core.h%'
        ORDER BY name
    ''')
    
    results = cursor.fetchall()
    for name, impl, tested, sig in results:
        status = 'COMPLETE' if impl and tested else 'IMPLEMENTED' if impl else 'TODO'
        print(f'  {status:10} {name}')
    
    conn.close()

if __name__ == "__main__":
    search_all_dependencies()