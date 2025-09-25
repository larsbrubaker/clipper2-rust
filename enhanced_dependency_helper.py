#!/usr/bin/env python3
"""
Enhanced Dependency Helper Tool for Clipper2 Library Port
Uses database dependency tracking for fast queries and implementation planning
"""

import sqlite3
import argparse
from pathlib import Path
from typing import List, Dict, Tuple
from collections import defaultdict

class EnhancedDependencyHelper:
    def __init__(self, db_path: str):
        self.db_path = db_path
        self.conn = sqlite3.connect(db_path)
        self.conn.row_factory = sqlite3.Row  # Enable column access by name
    
    def get_ready_functions(self, limit: int = 20) -> List[Dict]:
        """Get functions ready to implement using database"""
        cursor = self.conn.cursor()
        cursor.execute('''
            SELECT f.name, f.filepath, f.class_name, f.direct_dependency_count,
                   f.implementation_order, il.level_number,
                   CASE WHEN f.class_name THEN f.class_name || '::' || f.name 
                        ELSE f.name END as qualified_name
            FROM functions f
            LEFT JOIN implementation_levels il ON f.id = il.function_id
            WHERE f.ready_to_implement = 1 AND f.rust_implemented = 0
            ORDER BY f.implementation_order
            LIMIT ?
        ''', (limit,))
        
        results = []
        for row in cursor.fetchall():
            results.append({
                'name': row['qualified_name'],
                'simple_name': row['name'],
                'file': Path(row['filepath']).name if row['filepath'] else 'unknown',
                'dependency_count': row['direct_dependency_count'],
                'implementation_order': row['implementation_order'],
                'level': row['level_number']
            })
        
        return results
    
    def get_function_details(self, func_name: str) -> Dict:
        """Get comprehensive details about a function"""
        cursor = self.conn.cursor()
        
        # Find the function
        cursor.execute('''
            SELECT f.*, il.level_number, il.implementation_order as level_order
            FROM functions f
            LEFT JOIN implementation_levels il ON f.id = il.function_id
            WHERE f.name = ? OR (f.class_name || '::' || f.name) = ?
        ''', (func_name, func_name))
        
        row = cursor.fetchone()
        if not row:
            return {'error': f'Function {func_name} not found'}
        
        # Get dependencies
        cursor.execute('''
            SELECT dep.caller_name, dep.callee_name, dep.callee_implemented, dep.callee_tested
            FROM function_dependency_details dep
            WHERE dep.caller_qualified = ? OR dep.caller_name = ?
        ''', (func_name, func_name))
        
        dependencies = []
        for dep_row in cursor.fetchall():
            dependencies.append({
                'name': dep_row['callee_name'],
                'implemented': bool(dep_row['callee_implemented']),
                'tested': bool(dep_row['callee_tested']),
                'ready': bool(dep_row['callee_implemented']) and bool(dep_row['callee_tested'])
            })
        
        # Get dependents
        cursor.execute('''
            SELECT dep.caller_name, dep.caller_implemented, dep.caller_qualified
            FROM function_dependency_details dep
            WHERE dep.callee_qualified = ? OR dep.callee_name = ?
        ''', (func_name, func_name))
        
        dependents = []
        for dep_row in cursor.fetchall():
            dependents.append({
                'name': dep_row['caller_qualified'],
                'implemented': bool(dep_row['caller_implemented'])
            })
        
        return {
            'function': {
                'name': row['name'],
                'qualified_name': f"{row['class_name']}::{row['name']}" if row['class_name'] else row['name'],
                'filepath': row['filepath'],
                'line_number': row['line_number'],
                'class_name': row['class_name'],
                'rust_implemented': bool(row['rust_implemented']),
                'rust_tested': bool(row['rust_tested']),
                'implementation_order': row['implementation_order'],
                'level_number': row['level_number'],
                'ready_to_implement': bool(row['ready_to_implement']),
                'is_leaf_function': bool(row['is_leaf_function']),
                'complexity_score': row['complexity_score']
            },
            'dependencies': dependencies,
            'dependents': dependents,
            'stats': {
                'dependency_count': row['direct_dependency_count'],
                'dependent_count': row['dependent_count'],
                'is_leaf': bool(row['is_leaf_function']),
                'is_root': bool(row['is_root_function'])
            }
        }
    
    def get_implementation_progress(self) -> Dict:
        """Get overall implementation progress by dependency level"""
        cursor = self.conn.cursor()
        cursor.execute('SELECT * FROM implementation_progress ORDER BY level_number')
        
        levels = []
        total_stats = {'total': 0, 'implemented': 0, 'tested': 0, 'ready': 0}
        
        for row in cursor.fetchall():
            level_info = {
                'level': row['level_number'],
                'total_functions': row['total_functions'],
                'implemented': row['implemented'],
                'tested': row['tested'],
                'ready': row['ready'],
                'completion_pct': (row['implemented'] / row['total_functions'] * 100) if row['total_functions'] > 0 else 0
            }
            levels.append(level_info)
            
            total_stats['total'] += row['total_functions']
            total_stats['implemented'] += row['implemented']
            total_stats['tested'] += row['tested']
            total_stats['ready'] += row['ready']
        
        total_stats['completion_pct'] = (total_stats['implemented'] / total_stats['total'] * 100) if total_stats['total'] > 0 else 0
        
        return {
            'levels': levels,
            'totals': total_stats,
            'next_recommended_level': self._get_next_level_to_implement()
        }
    
    def _get_next_level_to_implement(self) -> int:
        """Find the next dependency level that should be worked on"""
        cursor = self.conn.cursor()
        cursor.execute('''
            SELECT level_number, COUNT(*) as total,
                   SUM(CASE WHEN f.rust_implemented = 1 THEN 1 ELSE 0 END) as done
            FROM implementation_levels il
            JOIN functions f ON il.function_id = f.id
            GROUP BY level_number
            HAVING done < total
            ORDER BY level_number
            LIMIT 1
        ''')
        
        row = cursor.fetchone()
        return row['level_number'] if row else -1
    
    def get_functions_by_level(self, level: int, limit: int = 50) -> List[Dict]:
        """Get all functions at a specific dependency level"""
        cursor = self.conn.cursor()
        cursor.execute('''
            SELECT f.name, f.filepath, f.class_name, f.rust_implemented, f.rust_tested,
                   f.ready_to_implement, f.direct_dependency_count,
                   CASE WHEN f.class_name THEN f.class_name || '::' || f.name 
                        ELSE f.name END as qualified_name
            FROM functions f
            JOIN implementation_levels il ON f.id = il.function_id
            WHERE il.level_number = ?
            ORDER BY f.implementation_order
            LIMIT ?
        ''', (level, limit))
        
        results = []
        for row in cursor.fetchall():
            status = 'READY' if row['ready_to_implement'] and not row['rust_implemented'] else \
                    ('DONE' if row['rust_implemented'] and row['rust_tested'] else \
                     ('IMPL' if row['rust_implemented'] else 'BLOCKED'))
                     
            results.append({
                'name': row['qualified_name'],
                'file': Path(row['filepath']).name if row['filepath'] else 'unknown',
                'status': status,
                'dependency_count': row['direct_dependency_count'],
                'implemented': bool(row['rust_implemented']),
                'tested': bool(row['rust_tested']),
                'ready': bool(row['ready_to_implement'])
            })
        
        return results
    
    def search_functions(self, pattern: str, limit: int = 20) -> List[Dict]:
        """Search functions with enhanced filtering"""
        cursor = self.conn.cursor()
        cursor.execute('''
            SELECT f.name, f.filepath, f.class_name, f.rust_implemented, f.rust_tested,
                   f.ready_to_implement, f.implementation_order, f.direct_dependency_count,
                   CASE WHEN f.class_name THEN f.class_name || '::' || f.name 
                        ELSE f.name END as qualified_name
            FROM functions f
            WHERE f.name LIKE ? OR (f.class_name || '::' || f.name) LIKE ?
               OR f.filepath LIKE ?
            ORDER BY f.implementation_order
            LIMIT ?
        ''', (f'%{pattern}%', f'%{pattern}%', f'%{pattern}%', limit))
        
        results = []
        for row in cursor.fetchall():
            status = 'READY' if row['ready_to_implement'] and not row['rust_implemented'] else \
                    ('DONE' if row['rust_implemented'] and row['rust_tested'] else \
                     ('IMPL' if row['rust_implemented'] else 'TODO'))
                     
            results.append({
                'name': row['qualified_name'],
                'file': Path(row['filepath']).name if row['filepath'] else 'unknown',
                'status': status,
                'order': row['implementation_order'],
                'deps': row['direct_dependency_count']
            })
        
        return results
    
    def get_blocking_functions(self, func_name: str) -> List[Dict]:
        """Find which functions are blocking this function's implementation"""
        cursor = self.conn.cursor()
        cursor.execute('''
            SELECT callee.name, callee.filepath, callee.class_name,
                   callee.rust_implemented, callee.rust_tested,
                   CASE WHEN callee.class_name THEN callee.class_name || '::' || callee.name 
                        ELSE callee.name END as qualified_name
            FROM function_dependencies fd
            JOIN functions caller ON fd.caller_id = caller.id
            JOIN functions callee ON fd.callee_id = callee.id
            WHERE (caller.name = ? OR (caller.class_name || '::' || caller.name) = ?)
              AND (callee.rust_implemented = 0 OR callee.rust_tested = 0)
            ORDER BY callee.implementation_order
        ''', (func_name, func_name))
        
        blocking = []
        for row in cursor.fetchall():
            blocking.append({
                'name': row['qualified_name'],
                'file': Path(row['filepath']).name if row['filepath'] else 'unknown',
                'implemented': bool(row['rust_implemented']),
                'tested': bool(row['rust_tested']),
                'issue': 'Not implemented' if not row['rust_implemented'] else 'Not tested'
            })
        
        return blocking
    
    def print_function_status(self, func_name: str):
        """Print comprehensive function status"""
        details = self.get_function_details(func_name)
        
        if 'error' in details:
            print(f"Error: {details['error']}")
            return
        
        func = details['function']
        stats = details['stats']
        
        print(f"\nFUNCTION: {func['qualified_name']}")
        print(f"File: {Path(func['filepath']).name}:{func['line_number']}")
        print(f"Implementation Order: #{func['implementation_order']} (Level {func['level_number']})")
        print(f"Status: {'[COMPLETE]' if func['rust_implemented'] and func['rust_tested'] else '[TODO]'}")
        print(f"  Implemented: {'YES' if func['rust_implemented'] else 'NO'}")
        print(f"  Tested: {'YES' if func['rust_tested'] else 'NO'}")
        print(f"Ready to implement: {'YES' if func['ready_to_implement'] else 'NO'}")
        print(f"Complexity Score: {func['complexity_score']:.1f}/10.0")
        func_type = 'Leaf (no deps)' if stats['is_leaf'] else f"{stats['dependency_count']} dependencies"
        print(f"Function Type: {func_type}")
        
        if details['dependencies']:
            print(f"\nDEPENDENCIES ({len(details['dependencies'])}):")
            for dep in details['dependencies']:
                status = "[READY]" if dep['ready'] else "[BLOCKED]"
                impl_test = f"I:{'Y' if dep['implemented'] else 'N'} T:{'Y' if dep['tested'] else 'N'}"
                print(f"  {status} {dep['name']} ({impl_test})")
        
        if details['dependents']:
            print(f"\nDEPENDENTS ({len(details['dependents'])}) - functions that need this one:")
            for dep in details['dependents'][:5]:  # Show first 5
                status = "[DONE]" if dep['implemented'] else "[WAITING]"
                print(f"  {status} {dep['name']}")
            if len(details['dependents']) > 5:
                print(f"  ... and {len(details['dependents']) - 5} more")
    
    def print_implementation_plan(self, level_limit: int = 5):
        """Print suggested implementation plan"""
        progress = self.get_implementation_progress()
        
        print("IMPLEMENTATION PROGRESS BY DEPENDENCY LEVEL")
        print("=" * 50)
        print(f"{'Level':<6} {'Total':<6} {'Done':<6} {'Ready':<6} {'%Done':<8}")
        print("-" * 50)
        
        for level in progress['levels'][:level_limit]:
            print(f"{level['level']:<6} {level['total_functions']:<6} {level['implemented']:<6} "
                  f"{level['ready']:<6} {level['completion_pct']:<8.1f}")
        
        totals = progress['totals']
        print("-" * 50)
        print(f"{'TOTAL':<6} {totals['total']:<6} {totals['implemented']:<6} "
              f"{totals['ready']:<6} {totals['completion_pct']:<8.1f}")
        
        next_level = progress['next_recommended_level']
        if next_level >= 0:
            print(f"\nRECOMMENDED: Focus on Level {next_level} next")
            ready_at_level = self.get_functions_by_level(next_level, 10)
            ready_funcs = [f for f in ready_at_level if f['status'] == 'READY']
            
            if ready_funcs:
                print(f"\nNext {len(ready_funcs)} functions to implement at Level {next_level}:")
                for i, func in enumerate(ready_funcs, 1):
                    print(f"  {i}. {func['name']} - {func['file']}")
    
    def close(self):
        self.conn.close()

def main():
    parser = argparse.ArgumentParser(description='Enhanced dependency helper using database')
    parser.add_argument('--ready', '-r', action='store_true', help='Show functions ready to implement')
    parser.add_argument('--function', '-f', help='Show detailed function status')
    parser.add_argument('--search', '-s', help='Search functions by pattern')
    parser.add_argument('--level', '-l', type=int, help='Show functions at specific dependency level')
    parser.add_argument('--progress', '-p', action='store_true', help='Show implementation progress')
    parser.add_argument('--blocking', '-b', help='Show what is blocking a function')
    parser.add_argument('--plan', action='store_true', help='Show implementation plan')
    parser.add_argument('--limit', type=int, default=20, help='Limit results')
    
    args = parser.parse_args()
    
    db_path = r"C:\Development\clipper2-rust\clipper2_complete.db"
    helper = EnhancedDependencyHelper(db_path)
    
    try:
        if args.ready:
            ready = helper.get_ready_functions(args.limit)
            print(f"FUNCTIONS READY TO IMPLEMENT ({len(ready)}):")
            print("=" * 60)
            for i, func in enumerate(ready, 1):
                print(f"{i:3d}. {func['name']}")
                print(f"     File: {func['file']} | Order: #{func['implementation_order']} | Level: {func['level']}")
                print(f"     Dependencies: {func['dependency_count']}")
                print()
                
        elif args.function:
            helper.print_function_status(args.function)
            
        elif args.search:
            results = helper.search_functions(args.search, args.limit)
            print(f"SEARCH RESULTS for '{args.search}' ({len(results)}):")
            print("=" * 60)
            for func in results:
                print(f"{func['status']:<6} #{func['order']:<4} {func['name']:<30} {func['file']:<20} (deps:{func['deps']})")
                
        elif args.level is not None:
            funcs = helper.get_functions_by_level(args.level, args.limit)
            print(f"FUNCTIONS AT DEPENDENCY LEVEL {args.level} ({len(funcs)}):")
            print("=" * 60)
            for func in funcs:
                print(f"{func['status']:<6} {func['name']:<35} {func['file']:<20} (deps:{func['dependency_count']})")
                
        elif args.blocking:
            blocking = helper.get_blocking_functions(args.blocking)
            if blocking:
                print(f"FUNCTIONS BLOCKING '{args.blocking}' ({len(blocking)}):")
                print("=" * 50)
                for func in blocking:
                    print(f"[BLOCKED] {func['name']} - {func['issue']}")
            else:
                print(f"[READY] No functions blocking '{args.blocking}' - ready to implement!")
                
        elif args.progress or args.plan:
            helper.print_implementation_plan()
            
        else:
            print("Enhanced Dependency Helper - Database Edition")
            print("\nUsage examples:")
            print("  python enhanced_dependency_helper.py --ready")
            print("  python enhanced_dependency_helper.py --function 'PointInPolygon'")
            print("  python enhanced_dependency_helper.py --level 0")
            print("  python enhanced_dependency_helper.py --plan")
            print("  python enhanced_dependency_helper.py --blocking 'SomeFunction'")
            
    finally:
        helper.close()

if __name__ == "__main__":
    main()