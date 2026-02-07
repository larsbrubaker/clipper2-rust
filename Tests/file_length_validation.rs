use std::fs;
use std::path::{Path, PathBuf};

const MAX_FILE_LENGTH: usize = 4000;

/// Walk directory recursively to find all files with specified extensions
fn find_files_with_extensions(dir: &Path, extensions: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // Skip target directory, hidden directories, and C++ source directory
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name == "target"
                        || dir_name.starts_with('.')
                        || dir_name == "CPP"
                        || dir_name == "pkg"
                        || dir_name == "Tests"
                    {
                        continue;
                    }
                }
                files.extend(find_files_with_extensions(&path, extensions));
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext) {
                    files.push(path);
                }
            }
        }
    }

    files
}

/// Count lines in a file
fn count_lines_in_file(path: &Path) -> Result<usize, std::io::Error> {
    let content = fs::read_to_string(path)?;
    Ok(content.lines().count())
}

/// Test that ensures all source files are under the specified line limit
#[test]
fn test_source_files_line_count_under_limit() {
    let project_root = std::env::current_dir().unwrap();
    let extensions = ["rs", "py", "md"];

    let files = find_files_with_extensions(&project_root, &extensions);
    let mut files_over_limit = Vec::new();

    for file_path in files {
        match count_lines_in_file(&file_path) {
            Ok(line_count) => {
                if line_count > MAX_FILE_LENGTH {
                    files_over_limit.push((file_path.clone(), line_count));
                }

                // Print file info for visibility during tests
                println!("File: {} - Lines: {}", file_path.display(), line_count);
            }
            Err(e) => {
                panic!("Failed to read file {}: {}", file_path.display(), e);
            }
        }
    }

    if !files_over_limit.is_empty() {
        let mut error_message = format!(
            "\nThe following {} file(s) exceed the {}-line limit and should be refactored:\n",
            files_over_limit.len(),
            MAX_FILE_LENGTH
        );

        for (path, lines) in &files_over_limit {
            error_message.push_str(&format!("  - {} ({} lines)\n", path.display(), lines));
        }

        error_message.push_str(&format!(
            "\nConsider breaking large files into smaller modules or functions.\n\
             Files with over {} lines are often harder to maintain and test.\n\
             This is especially important for the Clipper2 implementation which follows\n\
             a zero-tolerance policy for complexity.",
            MAX_FILE_LENGTH
        ));

        panic!("{}", error_message);
    }

    println!(
        "✅ All source files are within the {}-line limit!",
        MAX_FILE_LENGTH
    );
}

/// Test individual file types for more granular feedback
#[test]
fn test_rust_files_line_count() {
    let project_root = std::env::current_dir().unwrap();
    let rust_files = find_files_with_extensions(&project_root, &["rs"]);

    for file_path in rust_files {
        let line_count = count_lines_in_file(&file_path).unwrap();

        assert!(
            line_count <= MAX_FILE_LENGTH,
            "Rust file {} has {} lines, which exceeds the {}-line limit. \
             Consider refactoring into smaller modules. This is critical for the \
             Clipper2 implementation which requires maintainable, focused code.",
            file_path.display(),
            line_count,
            MAX_FILE_LENGTH
        );
    }
}

#[test]
fn test_python_files_line_count() {
    let project_root = std::env::current_dir().unwrap();
    let py_files = find_files_with_extensions(&project_root, &["py"]);

    for file_path in py_files {
        let line_count = count_lines_in_file(&file_path).unwrap();

        assert!(
            line_count <= MAX_FILE_LENGTH,
            "Python file {} has {} lines, which exceeds the {}-line limit. \
             Consider breaking into smaller modules or functions.",
            file_path.display(),
            line_count,
            MAX_FILE_LENGTH
        );
    }
}

#[test]
fn test_markdown_files_line_count() {
    let project_root = std::env::current_dir().unwrap();
    let md_files = find_files_with_extensions(&project_root, &["md"]);

    for file_path in md_files {
        let line_count = count_lines_in_file(&file_path).unwrap();

        assert!(
            line_count <= MAX_FILE_LENGTH,
            "Markdown file {} has {} lines, which exceeds the {}-line limit. \
             Consider splitting into multiple documents for better readability.",
            file_path.display(),
            line_count,
            MAX_FILE_LENGTH
        );
    }
}

/// Test that validates the overall project structure
#[test]
fn test_project_structure_health() {
    let project_root = std::env::current_dir().unwrap();
    let extensions = ["rs", "py", "md"];
    let files = find_files_with_extensions(&project_root, &extensions);

    // Ensure we have files to test
    assert!(!files.is_empty(), "No source files found to validate");

    // Calculate average file length
    let mut total_lines = 0;
    let mut file_count = 0;

    for file_path in &files {
        if let Ok(line_count) = count_lines_in_file(file_path) {
            total_lines += line_count;
            file_count += 1;
        }
    }

    let average_lines = if file_count > 0 {
        total_lines / file_count
    } else {
        0
    };

    println!("📊 Clipper2 Rust Project Statistics:");
    println!("  - Total files analyzed: {}", file_count);
    println!("  - Total lines of code: {}", total_lines);
    println!("  - Average lines per file: {}", average_lines);
    println!("  - Maximum allowed lines per file: {}", MAX_FILE_LENGTH);

    // Optional: Warning if average is getting too high
    if average_lines > MAX_FILE_LENGTH / 2 {
        println!("⚠️  Warning: Average file length ({} lines) is approaching the limit. Consider proactive refactoring.", average_lines);
    }

    // Additional check for Clipper2-specific concerns
    let rust_files: Vec<_> = files
        .iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
        .collect();

    println!("🦀 Rust-specific metrics:");
    println!("  - Rust source files: {}", rust_files.len());

    if rust_files.len() > 20 {
        println!("💡 Consider organizing Rust modules into subdirectories as the project grows");
    }
}

#[cfg(test)]
mod file_metrics {
    use super::*;

    /// Helper function to generate refactoring suggestions based on file type
    fn get_refactoring_suggestions(file_path: &Path, line_count: usize) -> String {
        let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match extension {
            "rs" => format!(
                "Rust file {} ({} lines) refactoring suggestions:\n\
                 - Split large functions into smaller, focused functions\n\
                 - Extract related functionality into separate modules\n\
                 - Consider using traits to separate interface from implementation\n\
                 - Move complex algorithms into helper structs or enums\n\
                 - For Clipper2: Ensure each module handles a single geometric operation\n\
                 - Follow the project's zero-tolerance policy for incomplete functions",
                file_path.display(),
                line_count
            ),
            "py" => format!(
                "Python file {} ({} lines) refactoring suggestions:\n\
                 - Break large functions into smaller, focused functions\n\
                 - Extract database operations into separate modules\n\
                 - Consider using classes for complex state management\n\
                 - Move verification logic into utility functions\n\
                 - For Clipper2: Keep database analysis scripts modular",
                file_path.display(),
                line_count
            ),
            "md" => format!(
                "Markdown file {} ({} lines) refactoring suggestions:\n\
                 - Split into multiple focused documentation files\n\
                 - Use cross-references between documents\n\
                 - Consider creating a table of contents\n\
                 - Break long sections into separate files\n\
                 - For Clipper2: Separate implementation rules from status tracking",
                file_path.display(),
                line_count
            ),
            _ => format!(
                "File {} ({} lines) should be refactored for better maintainability",
                file_path.display(),
                line_count
            ),
        }
    }

    #[test]
    fn generate_refactoring_report() {
        let project_root = std::env::current_dir().unwrap();
        let extensions = ["rs", "py", "md"];
        let files = find_files_with_extensions(&project_root, &extensions);

        let mut large_files = Vec::new();

        for file_path in files {
            if let Ok(line_count) = count_lines_in_file(&file_path) {
                if line_count > MAX_FILE_LENGTH {
                    large_files.push((file_path, line_count));
                }
            }
        }

        if !large_files.is_empty() {
            println!("\n🔧 CLIPPER2 REFACTORING REPORT");
            println!("=============================");

            for (path, lines) in large_files {
                println!("\n{}", get_refactoring_suggestions(&path, lines));
            }

            println!("\n💡 Clipper2-Specific Refactoring Guidelines:");
            println!("- Keep functions under 50 lines when possible");
            println!("- Each function must be complete before marking as implemented");
            println!("- Follow the zero-tolerance policy for stubs and todos");
            println!("- Maintain exact behavioral matching with C++ implementation");
            println!("- Use the Single Responsibility Principle rigorously");
            println!("- Database tracking functions should be in separate utilities");
        } else {
            println!("✅ No files need refactoring - all files are within limits!");
            println!("🎉 Clipper2 project maintains excellent code organization!");
        }
    }

    #[test]
    fn validate_clipper2_project_structure() {
        let project_root = std::env::current_dir().unwrap();

        // Check for essential project files
        let essential_files = ["Cargo.toml", "CLAUDE.md"];

        for file in essential_files {
            let file_path = project_root.join(file);
            assert!(
                file_path.exists(),
                "Essential Clipper2 file missing: {}. This indicates project structure issues.",
                file
            );
        }

        // Check for essential directories
        let essential_dirs = ["src", "tests", "examples", "benches"];

        for dir in essential_dirs {
            let dir_path = project_root.join(dir);
            assert!(
                dir_path.exists() && dir_path.is_dir(),
                "Essential Clipper2 directory missing: {}. This indicates project structure issues.",
                dir
            );
        }

        println!("✅ Clipper2 project structure is valid");
    }
}
