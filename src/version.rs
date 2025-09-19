//! Version information for Clipper2
//!
//! Direct port from clipper.version.h

/// Clipper2 library version string
pub const CLIPPER2_VERSION: &str = "1.5.4";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_string() {
        assert_eq!(CLIPPER2_VERSION, "1.5.4");
        // Version string is non-empty by definition - no need to test
    }

    #[test]
    fn test_version_format() {
        // Verify it follows semantic versioning pattern
        let parts: Vec<&str> = CLIPPER2_VERSION.split('.').collect();
        assert_eq!(parts.len(), 3);

        // Each part should be numeric
        for part in parts {
            assert!(
                part.parse::<u32>().is_ok(),
                "Version part {} should be numeric",
                part
            );
        }
    }
}
