# CPP Reference & Test Code

This directory contains the original C++ Clipper2 library source code (used as reference during porting) and ad-hoc C++ test programs for comparison debugging.

## Directory Structure

- `Clipper2Lib/` - Original C++ Clipper2 library (headers + source)
- `Tests/` - Test data files (Polygons.txt, Offsets.txt, Lines.txt, PolytreeHoleOwner*.txt)
- `*.cpp` - Ad-hoc test programs for debugging specific issues
- `*.bat` - Build scripts for compiling test programs

## Compiling C++ Test Programs on Windows

### Prerequisites
- Visual Studio with C++ build tools (tested with VS 2022 Community and VS 18/2025)
- The `vcvarsall.bat` script must be available

### From MSYS/Git Bash

MSYS bash mangles `/` flags (e.g., `/EHsc` becomes `C:/msys64/EHsc`), so use batch files or invoke `cmd.exe` directly:

```bash
# Run a batch file from MSYS bash
cmd.exe //c "C:\\Development\\clipper2-rust\\CPP\\build_holes7.bat"

# Run the compiled exe
/c/Development/clipper2-rust/CPP/test_holes7.exe
```

### Batch File Template

Create a `.bat` file like this:

```batch
@echo off
call "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvarsall.bat" x64 >nul 2>&1
cd /d C:\Development\clipper2-rust\CPP
echo Compiling...
cl /EHsc /std:c++17 /O2 /I Clipper2Lib\include test_name.cpp Clipper2Lib\src\clipper.engine.cpp Clipper2Lib\src\clipper.offset.cpp Clipper2Lib\src\clipper.rectclip.cpp /Fe:test_name.exe /nologo
if %errorlevel% neq 0 (
    echo COMPILATION FAILED
    exit /b 1
)
echo Running...
test_name.exe
```

Adjust the `vcvarsall.bat` path to match your VS installation:
- VS 2022: `C:\Program Files\Microsoft Visual Studio\2022\Community\...`
- VS 18/2025: `C:\Program Files\Microsoft Visual Studio\18\Community\...`

### Linking Pre-compiled Objects

To speed up recompilation, pre-compile the library objects once:

```batch
cl /EHsc /std:c++17 /O2 /I Clipper2Lib\include /c Clipper2Lib\src\clipper.engine.cpp /Fo:clipper.engine.obj
cl /EHsc /std:c++17 /O2 /I Clipper2Lib\include /c Clipper2Lib\src\clipper.offset.cpp /Fo:clipper.offset.obj
cl /EHsc /std:c++17 /O2 /I Clipper2Lib\include /c Clipper2Lib\src\clipper.rectclip.cpp /Fo:clipper.rectclip.obj
```

Then link against the pre-compiled objects:

```batch
cl /EHsc /std:c++17 /O2 /I Clipper2Lib\include /c test_name.cpp /Fo:test_name.obj
link test_name.obj clipper.engine.obj clipper.offset.obj clipper.rectclip.obj /OUT:test_name.exe
```

## Notes

- `CLIPPER2_HI_PRECISION` is OFF by default in C++ (matching our Rust port)
- Output from C++ tests can be compared against Rust test output to verify behavioral matching
