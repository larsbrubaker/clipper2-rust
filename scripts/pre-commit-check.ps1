# Pre-commit check script for Clipper2 Rust Port
# This script runs file length validation and other checks before commits

param(
    [switch]$Fix = $false
)

Write-Host "Running pre-commit checks for Clipper2 Rust Port..." -ForegroundColor Cyan

# Function to check if a command exists
function Test-Command($cmdname) {
    return [bool](Get-Command -Name $cmdname -ErrorAction SilentlyContinue)
}

# Check if cargo is available
if (!(Test-Command "cargo")) {
    Write-Host "Cargo is not installed or not in PATH" -ForegroundColor Red
    exit 1
}

$exitCode = 0

Write-Host "`nRunning file length validation..." -ForegroundColor Yellow
try {
    cargo test file_length_validation::test_source_files_line_count_under_limit --quiet
    if ($LASTEXITCODE -eq 0) {
        Write-Host "All files are within the 800-line limit" -ForegroundColor Green
    } else {
        Write-Host "Some files exceed the 800-line limit" -ForegroundColor Red
        Write-Host "   Run 'cargo test file_length_validation::file_metrics::generate_refactoring_report' for refactoring suggestions" -ForegroundColor Yellow
        $exitCode = 1
    }
} catch {
    Write-Host "File length validation failed: $_" -ForegroundColor Red
    $exitCode = 1
}

Write-Host "`nRunning unit tests..." -ForegroundColor Yellow
try {
    cargo test --lib --quiet
    if ($LASTEXITCODE -eq 0) {
        Write-Host "All unit tests passed" -ForegroundColor Green
    } else {
        Write-Host "Some unit tests failed" -ForegroundColor Red
        $exitCode = 1
    }
} catch {
    Write-Host "Unit tests failed: $_" -ForegroundColor Red
    $exitCode = 1
}

# Skip integration tests if they don't exist or only contain file_length_validation
if (Test-Path "tests") {
    $integrationTests = Get-ChildItem -Path "tests" -Filter "*.rs" | Where-Object { $_.Name -ne "file_length_validation.rs" }
    if ($integrationTests.Count -gt 0) {
        Write-Host "`nRunning integration tests..." -ForegroundColor Yellow
        try {
            cargo test --test "*" --quiet
            if ($LASTEXITCODE -eq 0) {
                Write-Host "Integration tests passed" -ForegroundColor Green
            } else {
                Write-Host "Integration tests failed" -ForegroundColor Red
                $exitCode = 1
            }
        } catch {
            Write-Host "Integration tests failed: $_" -ForegroundColor Red
            $exitCode = 1
        }
    }
}

if (Test-Command "cargo-fmt") {
    Write-Host "`nChecking code formatting..." -ForegroundColor Yellow
    try {
        cargo fmt --all -- --check
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Code formatting is correct" -ForegroundColor Green
        } else {
            Write-Host "Code formatting issues found" -ForegroundColor Red
            if ($Fix) {
                Write-Host "Fixing code formatting..." -ForegroundColor Yellow
                cargo fmt --all
                Write-Host "Code formatting fixed" -ForegroundColor Green
            } else {
                Write-Host "   Run 'cargo fmt --all' to fix formatting or use -Fix flag" -ForegroundColor Yellow
                $exitCode = 1
            }
        }
    } catch {
        Write-Host "Code formatting check failed: $_" -ForegroundColor Red
        $exitCode = 1
    }
} else {
    Write-Host "rustfmt not available, skipping formatting check" -ForegroundColor Yellow
}

if (Test-Command "cargo-clippy") {
    Write-Host "`nRunning clippy lints..." -ForegroundColor Yellow
    try {
        cargo clippy --all-targets --all-features -- -D warnings
        if ($LASTEXITCODE -eq 0) {
            Write-Host "No clippy warnings found" -ForegroundColor Green
        } else {
            Write-Host "Clippy warnings found" -ForegroundColor Red
            $exitCode = 1
        }
    } catch {
        Write-Host "Clippy check failed: $_" -ForegroundColor Red
        $exitCode = 1
    }
} else {
    Write-Host "clippy not available, skipping lint check" -ForegroundColor Yellow
}

Write-Host "`nRunning build check..." -ForegroundColor Yellow
try {
    cargo build --all-targets
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Build successful" -ForegroundColor Green
    } else {
        Write-Host "Build failed" -ForegroundColor Red
        $exitCode = 1
    }
} catch {
    Write-Host "Build check failed: $_" -ForegroundColor Red
    $exitCode = 1
}

# Check for benchmark build if benchmarks exist
if (Test-Path "benches") {
    Write-Host "`nRunning benchmark build check..." -ForegroundColor Yellow
    try {
        cargo bench --no-run
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Benchmark build successful" -ForegroundColor Green
        } else {
            Write-Host "Benchmark build failed" -ForegroundColor Red
            $exitCode = 1
        }
    } catch {
        Write-Host "Benchmark build check failed: $_" -ForegroundColor Red
        $exitCode = 1
    }
}

# Database verification check if function_verifier exists
if (Test-Path "function_verifier.py") {
    if (Test-Command "python") {
        Write-Host "`nRunning database verification..." -ForegroundColor Yellow
        try {
            python function_verifier.py
            if ($LASTEXITCODE -eq 0) {
                Write-Host "Database verification successful" -ForegroundColor Green
            } else {
                Write-Host "Database verification failed" -ForegroundColor Red
                $exitCode = 1
            }
        } catch {
            Write-Host "Database verification check failed: $_" -ForegroundColor Red
            $exitCode = 1
        }
    } else {
        Write-Host "Python not available, skipping database verification" -ForegroundColor Yellow
    }
}

Write-Host "`n" -NoNewline
if ($exitCode -eq 0) {
    Write-Host "All pre-commit checks passed!" -ForegroundColor Green
    Write-Host "   Your Clipper2 Rust code is ready for commit." -ForegroundColor Green
} else {
    Write-Host "Pre-commit checks failed!" -ForegroundColor Red
    Write-Host "   Please fix the issues above before committing." -ForegroundColor Red
    Write-Host "`nHelpful commands:" -ForegroundColor Cyan
    Write-Host "   cargo test --verbose                    - Run tests with detailed output" -ForegroundColor White
    Write-Host "   cargo fmt --all                         - Fix formatting issues" -ForegroundColor White
    Write-Host "   cargo clippy --fix --all-targets        - Fix clippy warnings automatically" -ForegroundColor White
    Write-Host "   cargo test file_length_validation       - Check file lengths" -ForegroundColor White
    Write-Host "   cargo bench --no-run                    - Build benchmarks without running" -ForegroundColor White
    Write-Host "   python function_verifier.py             - Verify function database completeness" -ForegroundColor White
}

exit $exitCode