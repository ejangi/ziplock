@echo off
setlocal enabledelayedexpansion

REM ZipLock Desktop Launch Script for Windows
REM This script launches the cross-platform ZipLock desktop application

REM Parse command line arguments
set NO_BUILD=false
set DEBUG=false

:parse_args
if "%~1"=="" goto args_done
if "%~1"=="--no-build" (
    set NO_BUILD=true
    shift
    goto parse_args
)
if "%~1"=="-n" (
    set NO_BUILD=true
    shift
    goto parse_args
)
if "%~1"=="--debug" (
    set DEBUG=true
    shift
    goto parse_args
)
if "%~1"=="-d" (
    set DEBUG=true
    shift
    goto parse_args
)
if "%~1"=="--help" goto show_help
if "%~1"=="-h" goto show_help
echo ERROR: Unknown option: %~1
echo Usage: %0 [--no-build^|-n] [--debug^|-d] [--help^|-h]
exit /b 1

:show_help
echo ZipLock Desktop Development Launcher
echo.
echo Usage: %0 [OPTIONS]
echo.
echo Options:
echo   --no-build, -n    Skip building and run with existing binary
echo   --debug, -d       Run with debug logging enabled
echo   --help, -h        Show this help message
echo.
echo This script builds and runs the cross-platform ZipLock desktop application.
echo Works on Windows with Rust toolchain installed.
exit /b 0

:args_done

REM Get project root (assume we're running from project root directory)
set PROJECT_ROOT=%CD%

echo ZipLock Desktop Development Launcher
=======================================

REM Define paths
set FRONTEND_BIN=%PROJECT_ROOT%\target\release\ziplock.exe
set SHARED_LIB_DIR=%PROJECT_ROOT%\target\release

if "%NO_BUILD%"=="true" (
    echo Skipping build --no-build specified

    REM Check if binary exists
    if not exist "%FRONTEND_BIN%" (
        echo ERROR: ZipLock binary not found: %FRONTEND_BIN%
        echo        Run without --no-build to build first
        exit /b 1
    )

    REM Check if shared library exists
    if not exist "%SHARED_LIB_DIR%\ziplock_shared.dll" (
        echo ERROR: Shared library not found in: %SHARED_LIB_DIR%
        echo        Run without --no-build to build first
        exit /b 1
    )

    echo Using existing binaries
) else (
    echo Building ZipLock unified application...
    cd /d "%PROJECT_ROOT%"

    REM Build shared library first
    echo    Building shared library...
    cargo build --release -p ziplock-shared --features c-api
    if errorlevel 1 (
        echo ERROR: Failed to build shared library
        exit /b 1
    )
    echo    Shared library built successfully

    REM Build the desktop application
    echo    Building desktop application...
    if exist "apps\desktop\Cargo.toml" (
        cargo build --release --bin ziplock --manifest-path apps\desktop\Cargo.toml
        if errorlevel 1 (
            echo ERROR: Failed to build ZipLock desktop application
            exit /b 1
        )
        echo    ZipLock desktop application built successfully
    ) else (
        echo ERROR: apps\desktop\Cargo.toml not found!
        echo        Current directory: %cd%
        exit /b 1
    )
)

REM Set up environment for FFI (Windows uses PATH for DLL loading)
set PATH=%SHARED_LIB_DIR%;%PATH%

if "%DEBUG%"=="true" (
    echo Enabling debug logging...
    set RUST_LOG=debug
    set ZIPLOCK_LOG_LEVEL=debug
) else (
    set RUST_LOG=info
    set ZIPLOCK_LOG_LEVEL=info
)

echo Starting ZipLock Desktop Application...
echo    Binary: %FRONTEND_BIN%
echo    Library path: %SHARED_LIB_DIR%
echo    Log level: %RUST_LOG%

REM Start the unified application
cd /d "%PROJECT_ROOT%"
if "%DEBUG%"=="true" (
    echo    Running in debug mode with verbose output
    "%FRONTEND_BIN%" --verbose
) else (
    "%FRONTEND_BIN%"
)

echo ZipLock closed
