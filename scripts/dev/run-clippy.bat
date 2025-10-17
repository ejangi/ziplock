@echo off
REM ZipLock Clippy Check - Quick linting check
REM This script runs only the Clippy checks from the GitHub workflow

REM Function to print output (Windows doesn't support colors by default, so using plain text)
set "STEP_PREFIX==> "
set "SUCCESS_PREFIX=v "
set "ERROR_PREFIX=x "

REM Check command line arguments
if /i "%1"=="--help" goto :show_help
if /i "%1"=="-h" goto :show_help

REM Check if we're in the project root
if not exist "Cargo.toml" goto :wrong_directory
if not exist "apps\desktop" goto :wrong_directory
if not exist "shared" goto :wrong_directory
goto :correct_directory

:wrong_directory
echo %ERROR_PREFIX%This script must be run from the ZipLock project root directory
exit /b 1

:correct_directory

REM Determine if we're running in fix mode
if /i "%1"=="--fix" goto :fix_mode
goto :check_mode

:fix_mode
echo %STEP_PREFIX%Running Clippy with automatic fixes...
set "FIX_FLAG=--fix --allow-dirty --allow-staged"
goto :start_checks

:check_mode
echo %STEP_PREFIX%Running Clippy linting checks...
set "FIX_FLAG="
goto :start_checks

:start_checks
echo.

REM Check if clippy is installed
rustup component list --installed | findstr /C:"clippy" >nul 2>&1
if errorlevel 1 (
    echo %STEP_PREFIX%Installing clippy...
    rustup component add clippy
    if errorlevel 1 (
        echo %ERROR_PREFIX%Failed to install clippy
        exit /b 1
    )
)

REM Shared library clippy check
echo %STEP_PREFIX%Checking shared library...
cargo clippy -p ziplock-shared %FIX_FLAG% --all-targets -- -D warnings -A clippy::uninlined-format-args -A unused-imports -A dead-code -A clippy::not-unsafe-ptr-arg-deref -A clippy::should-implement-trait -A unused-unsafe -A clippy::collapsible-str-replace -A clippy::new-without-default -A clippy::let-and-return -A clippy::needless-borrows-for-generic-args -A clippy::needless-range-loop -A clippy::unnecessary-map-or -A clippy::collapsible-if -A clippy::needless-late-init -A clippy::unnecessary-cast -A clippy::needless-borrow -A clippy::field-reassign-with-default -A clippy::overly-complex-bool-expr -A clippy::for-kv-map -A unused-variables -A unused-must-use -A clippy::useless-format -A clippy::items-after-test-module -A clippy::manual-flatten -A unused-mut -A clippy::ptr-arg
if errorlevel 1 (
    echo %ERROR_PREFIX%Shared library Clippy check failed
    exit /b 1
)
echo %SUCCESS_PREFIX%Shared library Clippy check passed
echo.

REM Application clippy check (iced-gui features only)
echo %STEP_PREFIX%Checking unified application ^(iced-gui features only^)...
cargo clippy -p ziplock-desktop --no-default-features --features "iced-gui,file-dialog" %FIX_FLAG% --all-targets -- -D warnings -A clippy::uninlined-format-args -A unused-imports -A dead-code -A clippy::not-unsafe-ptr-arg-deref -A clippy::should-implement-trait -A unused-unsafe -A clippy::collapsible-str-replace -A clippy::new-without-default -A clippy::let-and-return -A clippy::needless-borrows-for-generic-args -A clippy::needless-range-loop -A clippy::unnecessary-map-or -A clippy::collapsible-if -A clippy::needless-late-init -A clippy::unnecessary-cast -A clippy::needless-borrow -A clippy::field-reassign-with-default -A clippy::overly-complex-bool-expr -A clippy::for-kv-map -A unused-variables -A unused-must-use -A clippy::useless-format -A clippy::items-after-test-module -A clippy::manual-flatten -A unused-mut -A clippy::ptr-arg
if errorlevel 1 (
    echo %ERROR_PREFIX%Application Clippy check failed
    exit /b 1
)
echo %SUCCESS_PREFIX%Application Clippy check passed
echo.

echo %SUCCESS_PREFIX%All Clippy checks passed! 🎉

if /i "%1"=="--fix" (
    echo.
    echo %STEP_PREFIX%Clippy fixes have been applied.
    echo %STEP_PREFIX%Review the changes and commit them:
    echo   git diff
    echo   git add .
    echo   git commit -m "Fix clippy warnings"
)

exit /b 0

:show_help
echo Usage: %0 [OPTIONS]
echo.
echo Run Clippy linting checks ^(same as GitHub CI^)
echo.
echo Options:
echo   --fix             Run clippy with --fix to automatically fix issues
echo   --help, -h        Show this help message
echo.
echo Examples:
echo   %0                # Run clippy checks
echo   %0 --fix          # Run clippy with automatic fixes
echo.
exit /b 0
