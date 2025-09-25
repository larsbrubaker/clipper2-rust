#!/usr/bin/env python3
"""
Database Schema Migration: Add Dependency Tracking
Adds comprehensive dependency tracking to the Clipper2 database
"""

import sqlite3
import json
from pathlib import Path
from typing import Dict, List, Set
from collections import defaultdict

class DependencyDatabaseMigration:
    def __init__(self, db_path: str, analysis_file: str = "dependency_analysis.json"):
        self.db_path = db_path
        self.conn = sqlite3.connect(db_path)
        self.conn.execute("PRAGMA foreign_keys = ON")
        
        # Load dependency analysis if available
        self.dependency_graph = {}
        self.reverse_deps = {}
        self.implementation_order = []
        
        try:
            with open(analysis_file, 'r') as f:
                analysis = json.load(f)
                self.dependency_graph = analysis.get('dependency_graph', {})
                self.reverse_deps = analysis.get('reverse_dependencies', {})
                self.implementation_order = analysis.get('implementation_order', [])
                print(f"Loaded dependency analysis: {len(self.dependency_graph)} functions with dependencies")
        except FileNotFoundError:
            print(f"Warning: {analysis_file} not found. Will create empty dependency structure.")
    
    def create_dependency_schema(self):
        """Create new tables and columns for dependency tracking"""
        cursor = self.conn.cursor()
        
        print("Creating dependency tracking schema...")
        
        # 1. Create function_dependencies table for direct relationships
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS function_dependencies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                caller_id INTEGER NOT NULL,
                callee_id INTEGER NOT NULL,
                dependency_type TEXT DEFAULT 'direct',
                call_frequency INTEGER DEFAULT 1,
                is_critical BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (caller_id) REFERENCES functions(id) ON DELETE CASCADE,
                FOREIGN KEY (callee_id) REFERENCES functions(id) ON DELETE CASCADE,
                UNIQUE(caller_id, callee_id)
            )
        ''')
        
        # 2. Create implementation_levels table for batching
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS implementation_levels (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                function_id INTEGER NOT NULL,
                level_number INTEGER NOT NULL,
                implementation_order INTEGER,
                can_implement_parallel BOOLEAN DEFAULT TRUE,
                estimated_complexity TEXT DEFAULT 'medium',
                FOREIGN KEY (function_id) REFERENCES functions(id) ON DELETE CASCADE,
                UNIQUE(function_id)
            )
        ''')
        
        # 3. Add new columns to functions table for quick access
        dependency_columns = [
            ('implementation_order', 'INTEGER'),
            ('direct_dependency_count', 'INTEGER DEFAULT 0'),
            ('total_dependency_count', 'INTEGER DEFAULT 0'),  
            ('dependent_count', 'INTEGER DEFAULT 0'),
            ('dependency_depth', 'INTEGER DEFAULT 0'),
            ('is_leaf_function', 'BOOLEAN DEFAULT FALSE'),
            ('is_root_function', 'BOOLEAN DEFAULT FALSE'),
            ('ready_to_implement', 'BOOLEAN DEFAULT FALSE'),
            ('complexity_score', 'REAL DEFAULT 0.0'),
            ('dependency_updated_at', 'TIMESTAMP')
        ]
        
        for col_name, col_definition in dependency_columns:
            try:
                cursor.execute(f'ALTER TABLE functions ADD COLUMN {col_name} {col_definition}')
                print(f"Added column: {col_name}")
            except sqlite3.OperationalError as e:
                if "duplicate column name" in str(e):
                    print(f"Column {col_name} already exists")
                else:
                    print(f"Error adding column {col_name}: {e}")
        
        # 4. Create indexes for performance
        indexes = [
            'CREATE INDEX IF NOT EXISTS idx_func_deps_caller ON function_dependencies(caller_id)',
            'CREATE INDEX IF NOT EXISTS idx_func_deps_callee ON function_dependencies(callee_id)',
            'CREATE INDEX IF NOT EXISTS idx_impl_levels_level ON implementation_levels(level_number)',
            'CREATE INDEX IF NOT EXISTS idx_func_impl_order ON functions(implementation_order)',
            'CREATE INDEX IF NOT EXISTS idx_func_ready ON functions(ready_to_implement)',
            'CREATE INDEX IF NOT EXISTS idx_func_leaf ON functions(is_leaf_function)',
            'CREATE INDEX IF NOT EXISTS idx_func_deps_count ON functions(direct_dependency_count)'
        ]
        
        for idx_sql in indexes:
            cursor.execute(idx_sql)
            
        self.conn.commit()
        print("Dependency schema created successfully!")
    
    def get_function_id_mapping(self) -> Dict[str, int]:
        """Create mapping from function names to database IDs"""
        cursor = self.conn.cursor()
        cursor.execute('''
            SELECT id, name, class_name 
            FROM functions 
            ORDER BY id
        ''')
        
        name_to_id = {}
        for func_id, name, class_name in cursor.fetchall():
            # Store both simple name and qualified name
            name_to_id[name] = func_id
            if class_name:
                qualified_name = f"{class_name}::{name}"
                name_to_id[qualified_name] = func_id
        
        return name_to_id
    
    def populate_dependency_data(self):
        """Populate dependency tables with analyzed data"""
        print("Populating dependency data...")
        cursor = self.conn.cursor()
        
        # Get function ID mapping
        name_to_id = self.get_function_id_mapping()
        
        # Clear existing dependency data
        cursor.execute('DELETE FROM function_dependencies')
        cursor.execute('DELETE FROM implementation_levels')
        
        # 1. Populate function_dependencies table
        dependency_count = 0
        for caller_name, callees in self.dependency_graph.items():
            caller_id = name_to_id.get(caller_name)
            if not caller_id:
                continue
                
            for callee_name in callees:
                callee_id = name_to_id.get(callee_name)
                if callee_id:
                    cursor.execute('''
                        INSERT OR IGNORE INTO function_dependencies 
                        (caller_id, callee_id, dependency_type) 
                        VALUES (?, ?, 'direct')
                    ''', (caller_id, callee_id))
                    dependency_count += 1
        
        print(f"Inserted {dependency_count} function dependencies")
        
        # 2. Populate implementation_levels and update functions table
        level_groups = self._compute_implementation_levels()
        
        for level_num, function_names in level_groups.items():
            for func_name in function_names:
                func_id = name_to_id.get(func_name)
                if func_id:
                    # Insert into implementation_levels
                    cursor.execute('''
                        INSERT OR IGNORE INTO implementation_levels 
                        (function_id, level_number, implementation_order) 
                        VALUES (?, ?, ?)
                    ''', (func_id, level_num, self.implementation_order.index(func_name) + 1 if func_name in self.implementation_order else None))
        
        # 3. Update functions table with computed statistics
        self._update_function_statistics(cursor, name_to_id)
        
        self.conn.commit()
        print("Dependency data populated successfully!")
    
    def _compute_implementation_levels(self) -> Dict[int, List[str]]:
        """Group functions by dependency level for parallel implementation"""
        levels = defaultdict(list)
        visited = set()
        
        def get_level(func_name: str, memo: Dict[str, int] = {}) -> int:
            if func_name in memo:
                return memo[func_name]
            
            if func_name in visited:
                return 0  # Circular dependency, assign to level 0
            
            visited.add(func_name)
            dependencies = self.dependency_graph.get(func_name, [])
            
            if not dependencies:
                level = 0  # Leaf function
            else:
                max_dep_level = -1
                for dep in dependencies:
                    dep_level = get_level(dep, memo)
                    max_dep_level = max(max_dep_level, dep_level)
                level = max_dep_level + 1
            
            memo[func_name] = level
            visited.remove(func_name)
            return level
        
        # Compute levels for all functions
        for func_name in self.implementation_order:
            level = get_level(func_name)
            levels[level].append(func_name)
        
        return levels
    
    def _update_function_statistics(self, cursor: sqlite3.Cursor, name_to_id: Dict[str, int]):
        """Update functions table with dependency statistics"""
        print("Computing function statistics...")
        
        for func_name, func_id in name_to_id.items():
            # Skip duplicate mappings (simple name vs qualified name)
            if '::' in func_name:
                continue
                
            # Get qualified name if exists
            cursor.execute('SELECT name, class_name FROM functions WHERE id = ?', (func_id,))
            result = cursor.fetchone()
            if not result:
                continue
                
            name, class_name = result
            qualified_name = f"{class_name}::{name}" if class_name else name
            
            # Direct dependency count
            direct_deps = len(self.dependency_graph.get(qualified_name, []))
            
            # Dependent count (how many functions depend on this one)
            dependent_count = len(self.reverse_deps.get(qualified_name, []))
            
            # Is leaf function (no dependencies)
            is_leaf = direct_deps == 0
            
            # Is root function (nothing depends on it)
            is_root = dependent_count == 0
            
            # Implementation order
            impl_order = None
            if qualified_name in self.implementation_order:
                impl_order = self.implementation_order.index(qualified_name) + 1
            
            # Ready to implement check
            cursor.execute('''
                SELECT COUNT(*) FROM function_dependencies fd
                JOIN functions f ON fd.callee_id = f.id
                WHERE fd.caller_id = ? AND (f.rust_implemented = 0 OR f.rust_tested = 0)
            ''', (func_id,))
            unready_deps = cursor.fetchone()[0]
            ready_to_implement = unready_deps == 0
            
            # Complexity score (simple heuristic)
            complexity_score = min(10.0, direct_deps * 0.5 + dependent_count * 0.3)
            
            # Update the function record
            cursor.execute('''
                UPDATE functions SET 
                    direct_dependency_count = ?,
                    dependent_count = ?,
                    is_leaf_function = ?,
                    is_root_function = ?,
                    implementation_order = ?,
                    ready_to_implement = ?,
                    complexity_score = ?
                WHERE id = ?
            ''', (direct_deps, dependent_count, is_leaf, is_root, impl_order, 
                  ready_to_implement, complexity_score, func_id))
        
        print("Function statistics updated!")
    
    def create_dependency_views(self):
        """Create useful views for dependency queries"""
        cursor = self.conn.cursor()
        
        views = [
            # View for ready-to-implement functions
            '''
            CREATE VIEW IF NOT EXISTS ready_functions AS
            SELECT f.*, il.level_number
            FROM functions f
            LEFT JOIN implementation_levels il ON f.id = il.function_id
            WHERE f.rust_implemented = 0 AND f.ready_to_implement = 1
            ORDER BY f.implementation_order
            ''',
            
            # View for function dependency details
            '''
            CREATE VIEW IF NOT EXISTS function_dependency_details AS
            SELECT 
                caller.name as caller_name,
                CASE WHEN caller.class_name THEN caller.class_name || '::' || caller.name 
                     ELSE caller.name END as caller_qualified,
                callee.name as callee_name,
                CASE WHEN callee.class_name THEN callee.class_name || '::' || callee.name 
                     ELSE callee.name END as callee_qualified,
                fd.dependency_type,
                caller.rust_implemented as caller_implemented,
                callee.rust_implemented as callee_implemented,
                callee.rust_tested as callee_tested
            FROM function_dependencies fd
            JOIN functions caller ON fd.caller_id = caller.id
            JOIN functions callee ON fd.callee_id = callee.id
            ''',
            
            # View for implementation progress by level
            '''
            CREATE VIEW IF NOT EXISTS implementation_progress AS
            SELECT 
                il.level_number,
                COUNT(*) as total_functions,
                SUM(CASE WHEN f.rust_implemented = 1 THEN 1 ELSE 0 END) as implemented,
                SUM(CASE WHEN f.rust_tested = 1 THEN 1 ELSE 0 END) as tested,
                SUM(CASE WHEN f.ready_to_implement = 1 THEN 1 ELSE 0 END) as ready
            FROM implementation_levels il
            JOIN functions f ON il.function_id = f.id
            GROUP BY il.level_number
            ORDER BY il.level_number
            '''
        ]
        
        for view_sql in views:
            cursor.execute(view_sql)
        
        self.conn.commit()
        print("Dependency views created!")
    
    def verify_migration(self):
        """Verify the migration was successful"""
        cursor = self.conn.cursor()
        
        # Check table counts
        cursor.execute('SELECT COUNT(*) FROM functions')
        func_count = cursor.fetchone()[0]
        
        cursor.execute('SELECT COUNT(*) FROM function_dependencies')
        dep_count = cursor.fetchone()[0]
        
        cursor.execute('SELECT COUNT(*) FROM implementation_levels')
        level_count = cursor.fetchone()[0]
        
        cursor.execute('SELECT COUNT(*) FROM functions WHERE ready_to_implement = 1')
        ready_count = cursor.fetchone()[0]
        
        cursor.execute('SELECT COUNT(*) FROM functions WHERE is_leaf_function = 1')
        leaf_count = cursor.fetchone()[0]
        
        print(f"\nMIGRATION VERIFICATION:")
        print(f"  Total functions: {func_count}")
        print(f"  Function dependencies: {dep_count}")
        print(f"  Implementation levels: {level_count}")
        print(f"  Ready to implement: {ready_count}")
        print(f"  Leaf functions: {leaf_count}")
        
        # Show a few examples
        cursor.execute('''
            SELECT name, direct_dependency_count, ready_to_implement, implementation_order
            FROM functions 
            WHERE ready_to_implement = 1 
            ORDER BY implementation_order 
            LIMIT 5
        ''')
        
        print(f"\nFirst 5 ready functions:")
        for name, dep_count, ready, order in cursor.fetchall():
            print(f"  {order:3d}. {name} (deps: {dep_count})")
    
    def close(self):
        self.conn.close()

def main():
    db_path = r"C:\Development\clipper2-rust\clipper2_complete.db"
    analysis_file = "dependency_analysis.json"
    
    print("CLIPPER2 DEPENDENCY DATABASE MIGRATION")
    print("=" * 45)
    
    migrator = DependencyDatabaseMigration(db_path, analysis_file)
    
    try:
        # Step 1: Create schema
        migrator.create_dependency_schema()
        
        # Step 2: Populate data
        migrator.populate_dependency_data()
        
        # Step 3: Create views
        migrator.create_dependency_views()
        
        # Step 4: Verify
        migrator.verify_migration()
        
        print("\nMIGRATION COMPLETED SUCCESSFULLY!")
        print("\nNew capabilities:")
        print("  - Function dependency tracking")
        print("  - Implementation readiness checking")
        print("  - Bottom-up implementation ordering")
        print("  - Parallel implementation grouping")
        print("  - Comprehensive dependency statistics")
        
    except Exception as e:
        print(f"\nMIGRATION FAILED: {e}")
        raise
    finally:
        migrator.close()

if __name__ == "__main__":
    main()