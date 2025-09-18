#!/usr/bin/env python3
"""
Dependency Analysis for Clipper2 Library
Analyzes C++ header files to determine implementation order
"""

import sqlite3
from pathlib import Path

def analyze_core_headers():
    """Analyze the core Clipper2 header files for structure"""
    cpp_root = Path(r"C:\Development\clipper2-rust\CPP")
    headers_dir = cpp_root / "Clipper2Lib" / "include" / "clipper2"
    
    # Core headers in likely dependency order
    core_headers = [
        "clipper.version.h",     # Version info - no dependencies
        "clipper.core.h",        # Core types and basic structures  
        "clipper.export.h",      # Export declarations
        "clipper.engine.h",      # Main clipping engine
        "clipper.offset.h",      # Path offsetting
        "clipper.rectclip.h",    # Rectangle clipping
        "clipper.minkowski.h",   # Minkowski operations
        "clipper.h"             # Main header that includes others
    ]
    
    print("=== CORE HEADER ANALYSIS ===\n")
    
    for header in core_headers:
        header_path = headers_dir / header
        if header_path.exists():
            analyze_header_dependencies(header_path)
        else:
            print(f"WARNING: Header not found: {header}")

def analyze_header_dependencies(header_path):
    """Analyze a single header file"""
    print(f"FILE: {header_path.name}")
    print("=" * 50)
    
    try:
        with open(header_path, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()
    except Exception as e:
        print(f"ERROR reading file: {e}\n")
        return
    
    # Find includes
    includes = []
    for line in content.split('\n'):
        line = line.strip()
        if line.startswith('#include'):
            includes.append(line)
    
    print("INCLUDES:")
    for inc in includes:
        print(f"   {inc}")
    
    # Find major constructs
    classes = []
    structs = []
    enums = []
    functions = []
    
    lines = content.split('\n')
    for i, line in enumerate(lines):
        line = line.strip()
        
        # Classes
        if line.startswith('class ') and not line.startswith('class;'):
            class_name = line.split()[1].split(':')[0].split('{')[0].strip()
            classes.append((class_name, i+1))
            
        # Structs  
        elif line.startswith('struct ') and not line.startswith('struct;'):
            struct_name = line.split()[1].split(':')[0].split('{')[0].strip()
            structs.append((struct_name, i+1))
            
        # Enums
        elif line.startswith('enum '):
            if 'class' in line:
                enum_name = line.split('class')[1].split()[0].split(':')[0].split('{')[0].strip()
            else:
                enum_name = line.split()[1].split(':')[0].split('{')[0].strip()
            enums.append((enum_name, i+1))
            
    print(f"\nCLASSES ({len(classes)}):")
    for name, line_num in classes:
        print(f"   class {name} (line {line_num})")
        
    print(f"\nSTRUCTS ({len(structs)}):")
    for name, line_num in structs:
        print(f"   struct {name} (line {line_num})")
        
    print(f"\nENUMS ({len(enums)}):")
    for name, line_num in enums:
        print(f"   enum {name} (line {line_num})")
        
    print("\n" + "="*50 + "\n")

def create_implementation_plan():
    """Create implementation plan based on analysis"""
    print("RUST IMPLEMENTATION PLAN")
    print("="*50)
    
    plan = [
        ("clipper.version.h", [
            "Version constants and macros"
        ]),
        ("clipper.core.h", [
            "Basic Point and Rect structures",
            "Path and Paths type aliases", 
            "FillRule and ClipType enums",
            "Basic geometric functions",
            "Exception classes"
        ]),
        ("clipper.export.h", [
            "CRect structure",
            "RectClip64 class",
            "RectClipLines64 class",
            "Export utility functions"
        ]),
        ("clipper.engine.h", [
            "Vertex and OutPt structures",
            "OutRec and LocalMinima structures", 
            "PolyPath classes",
            "ClipperBase class",
            "Clipper64 and ClipperD classes",
            "Core clipping algorithms"
        ]),
        ("clipper.offset.h", [
            "Group class",
            "ClipperOffset class", 
            "Path offsetting algorithms"
        ]),
        ("clipper.rectclip.h", [
            "OutPt2 class",
            "RectClip64 implementation",
            "Rectangle clipping algorithms"
        ]),
        ("clipper.minkowski.h", [
            "Minkowski sum/difference functions"
        ])
    ]
    
    total_priority = 1
    
    for header, items in plan:
        print(f"\nFILE: {header}")
        print("-" * 30)
        for item in items:
            print(f"   {total_priority:2d}. {item}")
            total_priority += 1
    
    print(f"\nTotal implementation items: {total_priority - 1}")

def main():
    analyze_core_headers()
    create_implementation_plan()
    
    print("\nNEXT STEPS:")
    print("1. Create Rust project structure")
    print("2. Start with clipper.version.h - version constants")
    print("3. Implement core types from clipper.core.h")
    print("4. Build unit tests for each component")
    print("5. Progress incrementally through dependency chain")

if __name__ == "__main__":
    main()