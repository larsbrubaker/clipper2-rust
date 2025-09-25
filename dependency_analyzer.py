#!/usr/bin/env python3
"""
Dependency Tree Generator for Clipper2 Library Port
Analyzes C++ source to create function call dependency tree for bottom-up implementation
"""

import sqlite3
import re
import os
from pathlib import Path
from typing import Dict, List, Set, Tuple
from collections import defaultdict, deque
import json

class DependencyAnalyzer:
    def __init__(self, db_path: str, cpp_root: str):
        self.db_path = db_path
        self.cpp_root = Path(cpp_root)
        self.conn = sqlite3.connect(db_path)
        self.all_functions = {}
        self.function_calls = defaultdict(set)
        self.dependency_graph = defaultdict(set)  # function -> set of functions it depends on
        self.reverse_deps = defaultdict(set)      # function -> set of functions that depend on it
        
    def load_functions_from_db(self):
        """Load all function names and signatures from database"""
        cursor = self.conn.cursor()
        cursor.execute('''
            SELECT name, filepath, signature, return_type, class_name 
            FROM functions 
            ORDER BY name
        ''')
        
        for name, filepath, signature, return_type, class_name in cursor.fetchall():
            # Create qualified name if part of a class
            qualified_name = f"{class_name}::{name}" if class_name else name
            self.all_functions[qualified_name] = {
                'name': name,
                'filepath': filepath,
                'signature': signature,
                'return_type': return_type,
                'class_name': class_name,
                'qualified_name': qualified_name
            }
    
    def extract_function_calls(self):
        """Extract function calls from all C++ source files"""
        print("Extracting function calls from C++ source...")
        
        for cpp_file in self.cpp_root.rglob("*.cpp"):
            self._analyze_file_for_calls(cpp_file)
            
        for h_file in self.cpp_root.rglob("*.h"):
            self._analyze_file_for_calls(h_file)
    
    def _analyze_file_for_calls(self, filepath: Path):
        """Analyze a single file for function calls"""
        try:
            with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
        except Exception as e:
            print(f"Warning: Could not read {filepath}: {e}")
            return
            
        # Remove comments and strings to avoid false positives
        content = self._remove_comments_and_strings(content)
        
        # Find all function calls in this file
        current_function = None
        lines = content.split('\n')
        
        for line_num, line in enumerate(lines, 1):
            line = line.strip()
            
            # Try to identify current function context
            func_def_match = re.match(r'^.*?([a-zA-Z_]\w*::[a-zA-Z_]\w*|[a-zA-Z_]\w*)\s*\([^)]*\)\s*[{:]', line)
            if func_def_match:
                current_function = self._normalize_function_name(func_def_match.group(1))
                
            # Look for function calls in this line
            if current_function:
                self._extract_calls_from_line(line, current_function, str(filepath), line_num)
    
    def _remove_comments_and_strings(self, content: str) -> str:
        """Remove comments and string literals to avoid false positives"""
        # Remove single-line comments
        content = re.sub(r'//.*?$', '', content, flags=re.MULTILINE)
        
        # Remove multi-line comments
        content = re.sub(r'/\*.*?\*/', '', content, flags=re.DOTALL)
        
        # Remove string literals (simplified)
        content = re.sub(r'"[^"\\]*(\\.[^"\\]*)*"', '""', content)
        content = re.sub(r"'[^'\\]*(\\.[^'\\]*)*'", "''", content)
        
        return content
    
    def _extract_calls_from_line(self, line: str, current_function: str, filepath: str, line_num: int):
        """Extract function calls from a single line of code"""
        # Patterns for function calls
        patterns = [
            # Standard function calls: func_name(
            r'\b([a-zA-Z_]\w*)\s*\(',
            # Member function calls: obj.func_name(
            r'\.([a-zA-Z_]\w*)\s*\(',
            # Scoped calls: Class::func_name(
            r'([a-zA-Z_]\w*::[a-zA-Z_]\w*)\s*\(',
            # Pointer calls: ptr->func_name(
            r'->([a-zA-Z_]\w*)\s*\(',
        ]
        
        for pattern in patterns:
            matches = re.finditer(pattern, line)
            for match in matches:
                called_function = self._normalize_function_name(match.group(1))
                
                # Skip obvious non-function calls
                if self._is_likely_function_call(called_function, line):
                    self.function_calls[current_function].add(called_function)
                    self.dependency_graph[current_function].add(called_function)
                    self.reverse_deps[called_function].add(current_function)
    
    def _normalize_function_name(self, func_name: str) -> str:
        """Normalize function name for consistent matching"""
        # Remove template parameters and extra whitespace
        func_name = re.sub(r'<[^<>]*>', '', func_name)
        return func_name.strip()
    
    def _is_likely_function_call(self, name: str, line: str) -> bool:
        """Filter out obvious non-function calls"""
        # Skip control flow keywords
        keywords = {'if', 'while', 'for', 'switch', 'return', 'throw', 'catch', 
                   'sizeof', 'typeof', 'static_cast', 'dynamic_cast', 'const_cast', 'reinterpret_cast'}
        
        if name.lower() in keywords:
            return False
            
        # Skip macro-like patterns (all caps)
        if name.isupper() and len(name) > 2:
            return False
            
        # Skip very short names or numbers
        if len(name) < 2 or name.isdigit():
            return False
            
        return True
    
    def build_dependency_tree(self):
        """Build complete dependency tree with transitive dependencies"""
        print("Building dependency tree...")
        
        # First, filter to only include functions we actually have in our database
        known_functions = set(self.all_functions.keys())
        known_simple_names = {info['name'] for info in self.all_functions.values()}
        
        filtered_deps = defaultdict(set)
        for caller, callees in self.dependency_graph.items():
            if caller in known_functions or caller in known_simple_names:
                for callee in callees:
                    # Try to match by full name or simple name
                    if callee in known_functions:
                        filtered_deps[caller].add(callee)
                    elif callee in known_simple_names:
                        # Find the full qualified name
                        for qualified, info in self.all_functions.items():
                            if info['name'] == callee:
                                filtered_deps[caller].add(qualified)
                                break
        
        self.dependency_graph = filtered_deps
        
        # Rebuild reverse dependencies
        self.reverse_deps = defaultdict(set)
        for caller, callees in self.dependency_graph.items():
            for callee in callees:
                self.reverse_deps[callee].add(caller)
    
    def find_implementation_order(self) -> List[str]:
        """Find bottom-up implementation order using topological sort"""
        print("Computing bottom-up implementation order...")
        
        # Create a copy of the dependency graph for modification
        deps = {k: v.copy() for k, v in self.dependency_graph.items()}
        
        # Add all functions that don't have dependencies
        all_functions = set(self.all_functions.keys())
        for func in all_functions:
            if func not in deps:
                deps[func] = set()
        
        # Kahn's algorithm for topological sort
        in_degree = defaultdict(int)
        for func in all_functions:
            in_degree[func] = 0
            
        for caller, callees in deps.items():
            for callee in callees:
                in_degree[caller] += 1
        
        # Start with functions that have no dependencies
        queue = deque([func for func in all_functions if in_degree[func] == 0])
        result = []
        
        while queue:
            current = queue.popleft()
            result.append(current)
            
            # Reduce in-degree for functions that depend on current
            for dependent in self.reverse_deps.get(current, set()):
                in_degree[dependent] -= 1
                if in_degree[dependent] == 0:
                    queue.append(dependent)
        
        # Check for cycles
        if len(result) != len(all_functions):
            print(f"Warning: Found circular dependencies. {len(all_functions) - len(result)} functions involved.")
            remaining = all_functions - set(result)
            print(f"Functions in cycles: {list(remaining)[:10]}...")  # Show first 10
            result.extend(remaining)  # Add remaining functions anyway
        
        return result
    
    def generate_dependency_report(self, output_file: str = "dependency_analysis.json"):
        """Generate comprehensive dependency analysis report"""
        
        # Compute statistics
        total_functions = len(self.all_functions)
        functions_with_deps = len([f for f, deps in self.dependency_graph.items() if deps])
        avg_dependencies = sum(len(deps) for deps in self.dependency_graph.values()) / max(1, len(self.dependency_graph))
        
        # Find leaf functions (no dependencies)
        leaf_functions = [func for func in self.all_functions.keys() 
                         if not self.dependency_graph.get(func, set())]
        
        # Find root functions (no dependents)
        root_functions = [func for func in self.all_functions.keys()
                         if not self.reverse_deps.get(func, set())]
        
        # Get implementation order
        implementation_order = self.find_implementation_order()
        
        report = {
            'statistics': {
                'total_functions': total_functions,
                'functions_with_dependencies': functions_with_deps,
                'average_dependencies_per_function': round(avg_dependencies, 2),
                'leaf_functions_count': len(leaf_functions),
                'root_functions_count': len(root_functions)
            },
            'leaf_functions': leaf_functions[:20],  # First 20 functions with no dependencies
            'root_functions': root_functions[:20],   # First 20 functions nothing depends on
            'implementation_order': implementation_order,
            'dependency_graph': {k: list(v) for k, v in self.dependency_graph.items()},
            'reverse_dependencies': {k: list(v) for k, v in self.reverse_deps.items()}
        }
        
        with open(output_file, 'w') as f:
            json.dump(report, f, indent=2)
        
        print(f"\nDEPENDENCY ANALYSIS COMPLETE")
        print("=" * 50)
        print(f"Total functions: {total_functions}")
        print(f"Functions with dependencies: {functions_with_deps}")
        print(f"Average dependencies per function: {avg_dependencies:.1f}")
        print(f"Leaf functions (no dependencies): {len(leaf_functions)}")
        print(f"Root functions (nothing depends on them): {len(root_functions)}")
        print(f"\nReport saved to: {output_file}")
        
        return report
    
    def print_implementation_order(self, limit: int = 50):
        """Print the first functions to implement (bottom-up order)"""
        order = self.find_implementation_order()
        
        print(f"\nBOTTOM-UP IMPLEMENTATION ORDER (first {limit} functions):")
        print("=" * 60)
        
        for i, func_name in enumerate(order[:limit], 1):
            func_info = self.all_functions.get(func_name, {})
            file_name = Path(func_info.get('filepath', '')).name if func_info.get('filepath') else 'unknown'
            deps_count = len(self.dependency_graph.get(func_name, set()))
            
            print(f"{i:3d}. {func_name}")
            print(f"     File: {file_name}")
            print(f"     Dependencies: {deps_count}")
            if deps_count > 0 and deps_count < 5:
                deps = list(self.dependency_graph.get(func_name, set()))
                print(f"     Depends on: {', '.join(deps[:3])}")
            print()
    
    def save_implementation_order_to_file(self, filename: str = "implementation_order.txt"):
        """Save complete implementation order to a text file"""
        order = self.find_implementation_order()
        
        with open(filename, 'w') as f:
            f.write("# CLIPPER2 BOTTOM-UP IMPLEMENTATION ORDER\n")
            f.write("# Implement functions in this order to satisfy dependencies\n\n")
            
            for i, func_name in enumerate(order, 1):
                func_info = self.all_functions.get(func_name, {})
                file_name = Path(func_info.get('filepath', '')).name if func_info.get('filepath') else 'unknown'
                deps_count = len(self.dependency_graph.get(func_name, set()))
                
                f.write(f"{i:4d}. {func_name}\n")
                f.write(f"      File: {file_name}\n")
                f.write(f"      Dependencies: {deps_count}\n")
                
                if deps_count > 0:
                    deps = list(self.dependency_graph.get(func_name, set()))
                    f.write(f"      Depends on: {', '.join(deps[:5])}\n")
                f.write("\n")
        
        print(f"Complete implementation order saved to: {filename}")
    
    def close(self):
        self.conn.close()

def main():
    db_path = r"C:\Development\clipper2-rust\clipper2_complete.db"
    cpp_root = r"C:\Development\clipper2-rust\CPP"
    
    print("CLIPPER2 DEPENDENCY ANALYZER")
    print("=" * 40)
    
    analyzer = DependencyAnalyzer(db_path, cpp_root)
    
    # Step 1: Load functions from database
    print("Loading functions from database...")
    analyzer.load_functions_from_db()
    print(f"Loaded {len(analyzer.all_functions)} functions")
    
    # Step 2: Extract function calls from source
    analyzer.extract_function_calls()
    print(f"Found {len(analyzer.function_calls)} functions making calls")
    
    # Step 3: Build dependency tree
    analyzer.build_dependency_tree()
    
    # Step 4: Generate reports
    report = analyzer.generate_dependency_report()
    analyzer.print_implementation_order()
    analyzer.save_implementation_order_to_file()
    
    print("\nRECOMMENDATION:")
    print("Start implementing functions from implementation_order.txt")
    print("Begin with leaf functions (no dependencies) and work your way up")
    print("This ensures all dependencies are satisfied before implementation")
    
    analyzer.close()

if __name__ == "__main__":
    main()