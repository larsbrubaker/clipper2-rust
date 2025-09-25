#!/usr/bin/env python3
"""
Dependency Helper Tool for Clipper2 Library Port
Interactive tool to check function dependencies and implementation readiness
"""

import sqlite3
import json
import argparse
from pathlib import Path
from typing import Set, List, Dict

class DependencyHelper:
    def __init__(self, db_path: str, analysis_file: str = "dependency_analysis.json"):
        self.db_path = db_path
        self.conn = sqlite3.connect(db_path)
        
        # Load dependency analysis
        try:
            with open(analysis_file, 'r') as f:
                self.analysis = json.load(f)
            self.dependency_graph = self.analysis.get('dependency_graph', {})
            self.reverse_deps = self.analysis.get('reverse_dependencies', {})
        except FileNotFoundError:
            print(f"Warning: {analysis_file} not found. Run dependency_analyzer.py first.")
            self.analysis = {}
            self.dependency_graph = {}
            self.reverse_deps = {}
    
    def get_function_info(self, func_name: str) -> Dict:
        """Get detailed information about a function"""
        cursor = self.conn.cursor()
        cursor.execute('''
            SELECT name, filepath, line_number, signature, return_type, 
                   class_name, rust_implemented, rust_tested
            FROM functions 
            WHERE name = ? OR (class_name || '::' || name) = ?
        ''', (func_name, func_name))
        
        result = cursor.fetchone()
        if result:
            return {
                'name': result[0],
                'filepath': result[1],
                'line_number': result[2],
                'signature': result[3],
                'return_type': result[4],
                'class_name': result[5],
                'qualified_name': f"{result[5]}::{result[0]}" if result[5] else result[0],
                'rust_implemented': bool(result[6]),
                'rust_tested': bool(result[7])
            }
        return {}
    
    def check_dependencies(self, func_name: str) -> Dict:
        """Check if all dependencies of a function are implemented"""
        func_info = self.get_function_info(func_name)
        if not func_info:
            return {'error': f'Function {func_name} not found'}
        
        qualified_name = func_info['qualified_name']
        dependencies = self.dependency_graph.get(qualified_name, [])
        
        dep_status = []
        all_ready = True
        
        for dep in dependencies:
            dep_info = self.get_function_info(dep)
            if dep_info:
                is_ready = dep_info['rust_implemented'] and dep_info['rust_tested']
                dep_status.append({
                    'name': dep,
                    'implemented': dep_info['rust_implemented'],
                    'tested': dep_info['rust_tested'],
                    'ready': is_ready,
                    'file': Path(dep_info['filepath']).name if dep_info['filepath'] else 'unknown'
                })
                if not is_ready:
                    all_ready = False
            else:
                # Dependency not found in database - might be external
                dep_status.append({
                    'name': dep,
                    'implemented': None,
                    'tested': None,
                    'ready': None,
                    'file': 'external/unknown'
                })
        
        return {
            'function': func_info,
            'dependencies': dep_status,
            'dependency_count': len(dependencies),
            'ready_to_implement': all_ready,
            'can_implement': all_ready and not func_info['rust_implemented']
        }
    
    def get_ready_functions(self, limit: int = 20) -> List[Dict]:
        """Get functions that are ready to implement (all dependencies satisfied)"""
        cursor = self.conn.cursor()
        cursor.execute('''
            SELECT name, filepath, class_name, rust_implemented, rust_tested
            FROM functions 
            WHERE rust_implemented = 0
            ORDER BY name
        ''')
        
        ready_functions = []
        
        for name, filepath, class_name, implemented, tested in cursor.fetchall():
            qualified_name = f"{class_name}::{name}" if class_name else name
            
            # Check if all dependencies are satisfied
            dependencies = self.dependency_graph.get(qualified_name, [])
            
            all_deps_ready = True
            for dep in dependencies:
                dep_info = self.get_function_info(dep)
                if dep_info and not (dep_info['rust_implemented'] and dep_info['rust_tested']):
                    all_deps_ready = False
                    break
            
            if all_deps_ready:
                ready_functions.append({
                    'name': qualified_name,
                    'file': Path(filepath).name if filepath else 'unknown',
                    'dependency_count': len(dependencies)
                })
                
                if len(ready_functions) >= limit:
                    break
        
        return ready_functions
    
    def get_dependents(self, func_name: str) -> List[Dict]:
        """Get functions that depend on this function"""
        func_info = self.get_function_info(func_name)
        if not func_info:
            return []
        
        qualified_name = func_info['qualified_name']
        dependents = self.reverse_deps.get(qualified_name, [])
        
        dependent_info = []
        for dep in dependents:
            dep_info = self.get_function_info(dep)
            if dep_info:
                dependent_info.append({
                    'name': dep,
                    'file': Path(dep_info['filepath']).name if dep_info['filepath'] else 'unknown',
                    'implemented': dep_info['rust_implemented'],
                    'tested': dep_info['rust_tested']
                })
        
        return dependent_info
    
    def search_functions(self, pattern: str) -> List[Dict]:
        """Search for functions by name pattern"""
        cursor = self.conn.cursor()
        cursor.execute('''
            SELECT name, filepath, class_name, rust_implemented, rust_tested
            FROM functions 
            WHERE name LIKE ? OR (class_name || '::' || name) LIKE ?
            ORDER BY name
            LIMIT 20
        ''', (f'%{pattern}%', f'%{pattern}%'))
        
        results = []
        for name, filepath, class_name, implemented, tested in cursor.fetchall():
            qualified_name = f"{class_name}::{name}" if class_name else name
            results.append({
                'name': qualified_name,
                'file': Path(filepath).name if filepath else 'unknown',
                'implemented': bool(implemented),
                'tested': bool(tested),
                'status': 'READY' if implemented and tested else ('IMPL' if implemented else 'TODO')
            })
        
        return results
    
    def print_function_status(self, func_name: str):
        """Print detailed status of a function"""
        status = self.check_dependencies(func_name)
        
        if 'error' in status:
            print(f"Error: {status['error']}")
            return
        
        func = status['function']
        print(f"\nFUNCTION: {func['qualified_name']}")
        print(f"File: {Path(func['filepath']).name}:{func['line_number']}")
        print(f"Implemented: {'✓' if func['rust_implemented'] else '✗'}")
        print(f"Tested: {'✓' if func['rust_tested'] else '✗'}")
        print(f"Ready to implement: {'✓' if status['ready_to_implement'] else '✗'}")
        
        if status['dependencies']:
            print(f"\nDEPENDENCIES ({len(status['dependencies'])}):")
            for dep in status['dependencies']:
                if dep['ready'] is None:
                    status_icon = "?"  # External dependency
                elif dep['ready']:
                    status_icon = "✓"
                else:
                    status_icon = "✗"
                
                impl_icon = "✓" if dep['implemented'] else ("?" if dep['implemented'] is None else "✗")
                test_icon = "✓" if dep['tested'] else ("?" if dep['tested'] is None else "✗")
                
                print(f"  {status_icon} {dep['name']} (I:{impl_icon} T:{test_icon}) - {dep['file']}")
        else:
            print("\nNo dependencies found - ready to implement!")
    
    def close(self):
        self.conn.close()

def main():
    parser = argparse.ArgumentParser(description='Check function dependencies for implementation')
    parser.add_argument('--function', '-f', help='Check specific function')
    parser.add_argument('--ready', '-r', action='store_true', help='Show functions ready to implement')
    parser.add_argument('--search', '-s', help='Search for functions by pattern')
    parser.add_argument('--dependents', '-d', help='Show what depends on this function')
    parser.add_argument('--limit', '-l', type=int, default=20, help='Limit number of results')
    
    args = parser.parse_args()
    
    db_path = r"C:\Development\clipper2-rust\clipper2_complete.db"
    helper = DependencyHelper(db_path)
    
    if args.function:
        helper.print_function_status(args.function)
        
    elif args.ready:
        ready = helper.get_ready_functions(args.limit)
        print(f"\nFUNCTIONS READY TO IMPLEMENT ({len(ready)}):")
        print("=" * 50)
        for i, func in enumerate(ready, 1):
            print(f"{i:2d}. {func['name']}")
            print(f"    File: {func['file']}")
            print(f"    Dependencies: {func['dependency_count']}")
            print()
            
    elif args.search:
        results = helper.search_functions(args.search)
        print(f"\nSEARCH RESULTS for '{args.search}':")
        print("=" * 50)
        for func in results:
            print(f"{func['status']} {func['name']} - {func['file']}")
            
    elif args.dependents:
        dependents = helper.get_dependents(args.dependents)
        print(f"\nFUNCTIONS DEPENDING ON '{args.dependents}':")
        print("=" * 50)
        for dep in dependents:
            status = "DONE" if dep['implemented'] and dep['tested'] else ("IMPL" if dep['implemented'] else "TODO")
            print(f"{status} {dep['name']} - {dep['file']}")
            
    else:
        print("Use --help for usage information")
        print("\nQuick examples:")
        print("  python dependency_helper.py --ready")
        print("  python dependency_helper.py --function 'PointInPolygon'")
        print("  python dependency_helper.py --search 'Point'")
    
    helper.close()

if __name__ == "__main__":
    main()