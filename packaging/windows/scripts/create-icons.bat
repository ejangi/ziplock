@echo off
REM ZipLock Windows Icon Generation Batch Script
REM Simple fallback method for creating .ico files

setlocal EnableDelayedExpansion

echo ZipLock Windows Icon Generation (Batch)
echo =======================================

REM Get script directory and project root
set "SCRIPT_DIR=%~dp0"
for %%i in ("%SCRIPT_DIR%..\..\..") do set "PROJECT_ROOT=%%~fi"
set "ASSETS_DIR=%PROJECT_ROOT%\assets\icons"
set "OUTPUT_DIR=%PROJECT_ROOT%\packaging\windows\resources"

echo Project Root: %PROJECT_ROOT%
echo Assets Directory: %ASSETS_DIR%
echo Output Directory: %OUTPUT_DIR%
echo.

REM Create output directory if it doesn't exist
if not exist "%OUTPUT_DIR%" (
    echo Creating output directory...
    mkdir "%OUTPUT_DIR%" 2>nul
)

REM Check if source files exist
if not exist "%ASSETS_DIR%\ziplock-icon-256.png" (
    echo ERROR: Source file not found: ziplock-icon-256.png
    goto :error
)

if not exist "%ASSETS_DIR%\ziplock-icon-128.png" (
    echo ERROR: Source file not found: ziplock-icon-128.png
    goto :error
)

if not exist "%ASSETS_DIR%\ziplock-icon-512.png" (
    echo ERROR: Source file not found: ziplock-icon-512.png
    goto :error
)

echo Copying PNG files as .ico (fallback method)...

REM Copy PNG files as .ico files (Windows can often handle this)
copy "%ASSETS_DIR%\ziplock-icon-256.png" "%OUTPUT_DIR%\ziplock.ico" >nul
if errorlevel 1 (
    echo ERROR: Failed to copy ziplock-icon-256.png to ziplock.ico
    goto :error
) else (
    echo   SUCCESS: Created ziplock.ico
)

copy "%ASSETS_DIR%\ziplock-icon-128.png" "%OUTPUT_DIR%\ziplock-small.ico" >nul
if errorlevel 1 (
    echo ERROR: Failed to copy ziplock-icon-128.png to ziplock-small.ico
    goto :error
) else (
    echo   SUCCESS: Created ziplock-small.ico
)

copy "%ASSETS_DIR%\ziplock-icon-512.png" "%OUTPUT_DIR%\ziplock-large.ico" >nul
if errorlevel 1 (
    echo ERROR: Failed to copy ziplock-icon-512.png to ziplock-large.ico
    goto :error
) else (
    echo   SUCCESS: Created ziplock-large.ico
)

echo.
echo Icon Generation Summary
echo ======================
dir "%OUTPUT_DIR%\*.ico" /b 2>nul | find /c /v "" > temp_count.txt
set /p ICON_COUNT=<temp_count.txt
del temp_count.txt 2>nul

echo Generated: %ICON_COUNT% icon files
echo Output directory: %OUTPUT_DIR%
echo.

if exist "%OUTPUT_DIR%\ziplock.ico" (
    for %%F in ("%OUTPUT_DIR%\ziplock.ico") do (
        set "SIZE=%%~zF"
        set /a "SIZE_KB=!SIZE!/1024"
        echo   - ziplock.ico (!SIZE_KB! KB)
    )
)

if exist "%OUTPUT_DIR%\ziplock-small.ico" (
    for %%F in ("%OUTPUT_DIR%\ziplock-small.ico") do (
        set "SIZE=%%~zF"
        set /a "SIZE_KB=!SIZE!/1024"
        echo   - ziplock-small.ico (!SIZE_KB! KB)
    )
)

if exist "%OUTPUT_DIR%\ziplock-large.ico" (
    for %%F in ("%OUTPUT_DIR%\ziplock-large.ico") do (
        set "SIZE=%%~zF"
        set /a "SIZE_KB=!SIZE!/1024"
        echo   - ziplock-large.ico (!SIZE_KB! KB)
    )
)

echo.
echo Next steps:
echo   1. The build.rs script will automatically embed these icons
echo   2. Build the Windows executable with: cargo build --release --target x86_64-pc-windows-msvc
echo   3. Test icon embedding with MSI installer
echo.
echo Icon generation completed successfully!
goto :end

:error
echo.
echo Icon generation failed!
echo Check that PNG source files exist in: %ASSETS_DIR%
exit /b 1

:end
endlocal
