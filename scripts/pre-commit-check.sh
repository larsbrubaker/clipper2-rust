#!/bin/bash
# Pre-commit check script for Clipper2 Rust Port
# This script runs file length validation and other checks before commits

set -e

FIX=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --fix)
            FIX=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--fix]"
            exit 1
            ;;
    esac
done

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

echo -e "${CYAN}🔍 Running pre-commit checks for Clipper2 Rust Port...${NC}"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if cargo is available
if ! command_exists cargo; then
    echo -e "${RED}❌ Cargo is not installed or not in PATH${NC}"
    exit 1
fi

EXIT_CODE=0

echo -e "\n${YELLOW}📏 Running file length validation...${NC}"
if cargo test file_length_validation::test_source_files_line_count_under_limit --quiet; then
    echo -e "${GREEN}✅ All files are within the 800-line limit${NC}"
else
    echo -e "${RED}❌ Some files exceed the 800-line limit${NC}"
    echo -e "${YELLOW}   Run 'cargo test file_length_validation::file_metrics::generate_refactoring_report' for refactoring suggestions${NC}"
    EXIT_CODE=1
fi

echo -e "\n${YELLOW}🧪 Running unit tests...${NC}"
if cargo test --lib --quiet; then
    echo -e "${GREEN}✅ All unit tests passed${NC}"
else
    echo -e "${RED}❌ Some unit tests failed${NC}"
    EXIT_CODE=1
fi

# Skip integration tests if they don't exist
if [ -d "tests" ] && find tests -name "*.rs" -not -name "file_length_validation.rs" | grep -q .; then
    echo -e "\n${YELLOW}🔗 Running integration tests...${NC}"
    if cargo test --test "*" --quiet; then
        echo -e "${GREEN}✅ Integration tests passed${NC}"
    else
        echo -e "${RED}❌ Integration tests failed${NC}"
        EXIT_CODE=1
    fi
fi

if command_exists rustfmt; then
    echo -e "\n${YELLOW}📝 Checking code formatting...${NC}"
    if cargo fmt --all -- --check; then
        echo -e "${GREEN}✅ Code formatting is correct${NC}"
    else
        echo -e "${RED}❌ Code formatting issues found${NC}"
        if [ "$FIX" = true ]; then
            echo -e "${YELLOW}🔧 Fixing code formatting...${NC}"
            cargo fmt --all
            echo -e "${GREEN}✅ Code formatting fixed${NC}"
        else
            echo -e "${YELLOW}   Run 'cargo fmt --all' to fix formatting or use --fix flag${NC}"
            EXIT_CODE=1
        fi
    fi
else
    echo -e "\n${YELLOW}⚠️  rustfmt not available, skipping formatting check${NC}"
fi

if command_exists cargo-clippy || cargo clippy --version >/dev/null 2>&1; then
    echo -e "\n${YELLOW}🔍 Running clippy lints...${NC}"
    if cargo clippy --all-targets --all-features -- -D warnings; then
        echo -e "${GREEN}✅ No clippy warnings found${NC}"
    else
        echo -e "${RED}❌ Clippy warnings found${NC}"
        EXIT_CODE=1
    fi
else
    echo -e "\n${YELLOW}⚠️  clippy not available, skipping lint check${NC}"
fi

echo -e "\n${YELLOW}🏗️  Running build check...${NC}"
if cargo build --all-targets; then
    echo -e "${GREEN}✅ Build successful${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    EXIT_CODE=1
fi

# Check for benchmark build if benchmarks exist
if [ -d "benches" ]; then
    echo -e "\n${YELLOW}🏃 Running benchmark build check...${NC}"
    if cargo bench --no-run; then
        echo -e "${GREEN}✅ Benchmark build successful${NC}"
    else
        echo -e "${RED}❌ Benchmark build failed${NC}"
        EXIT_CODE=1
    fi
fi

# Database verification check if function_verifier exists
if [ -f "function_verifier.py" ]; then
    if command_exists python; then
        echo -e "\n${YELLOW}🗄️  Running database verification...${NC}"
        if python function_verifier.py; then
            echo -e "${GREEN}✅ Database verification successful${NC}"
        else
            echo -e "${RED}❌ Database verification failed${NC}"
            EXIT_CODE=1
        fi
    else
        echo -e "\n${YELLOW}⚠️  Python not available, skipping database verification${NC}"
    fi
fi

echo
if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}🎉 All pre-commit checks passed!${NC}"
    echo -e "${GREEN}   Your Clipper2 Rust code is ready for commit.${NC}"
else
    echo -e "${RED}💥 Pre-commit checks failed!${NC}"
    echo -e "${RED}   Please fix the issues above before committing.${NC}"
    echo -e "\n${CYAN}📚 Helpful commands:${NC}"
    echo -e "${WHITE}   • cargo test --verbose                    - Run tests with detailed output${NC}"
    echo -e "${WHITE}   • cargo fmt --all                         - Fix formatting issues${NC}"
    echo -e "${WHITE}   • cargo clippy --fix --all-targets        - Fix clippy warnings automatically${NC}"
    echo -e "${WHITE}   • cargo test file_length_validation       - Check file lengths${NC}"
    echo -e "${WHITE}   • cargo bench --no-run                    - Build benchmarks without running${NC}"
    echo -e "${WHITE}   • python function_verifier.py             - Verify function database completeness${NC}"
fi

exit $EXIT_CODE