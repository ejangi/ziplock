@echo off
setlocal EnableDelayedExpansion

REM ZipLock Windows Archive Diagnostic Batch Script
REM Simple tool to examine archive contents and diagnose the metadata/credential mismatch

echo ============================================
echo ZipLock Windows Archive Diagnostic Tool
echo ============================================

if "%1"=="" (
    echo Usage: %0 ^<archive_path^> [password]
    echo Example: %0 "C:\Users\user\Downloads\ZipLock.7z"
    exit /b 1
)

set "ARCHIVE_PATH=%~1"
set "PASSWORD=%2"

echo Archive: %ARCHIVE_PATH%
if not exist "%ARCHIVE_PATH%" (
    echo ERROR: Archive file not found: %ARCHIVE_PATH%
    exit /b 1
)

REM Get archive size
for %%A in ("%ARCHIVE_PATH%") do set "ARCHIVE_SIZE=%%~zA"
echo Size: %ARCHIVE_SIZE% bytes

if "%PASSWORD%"=="" (
    echo Password: [NONE]
) else (
    echo Password: [PROVIDED]
)

REM Find 7z executable
set "SEVENZ="
if exist "C:\Program Files\7-Zip\7z.exe" (
    set "SEVENZ="C:\Program Files\7-Zip\7z.exe""
    echo Found 7z at: C:\Program Files\7-Zip\7z.exe
) else if exist "C:\Program Files (x86)\7-Zip\7z.exe" (
    set "SEVENZ="C:\Program Files (x86)\7-Zip\7z.exe""
    echo Found 7z at: C:\Program Files (x86)\7-Zip\7z.exe
) else (
    where 7z >nul 2>&1
    if !errorlevel! equ 0 (
        set "SEVENZ=7z"
        echo Found 7z in PATH
    ) else (
        echo ERROR: 7z executable not found
        echo Please install 7-Zip from: https://www.7-zip.org/
        exit /b 1
    )
)

echo.
echo ============================================
echo LISTING ARCHIVE CONTENTS
echo ============================================

REM List archive contents
if "%PASSWORD%"=="" (
    %SEVENZ% l "%ARCHIVE_PATH%"
) else (
    %SEVENZ% l "%ARCHIVE_PATH%" -p%PASSWORD%
)

if %errorlevel% neq 0 (
    echo ERROR: Failed to list archive contents
    if "%PASSWORD%"=="" (
        echo Try again with a password if the archive is encrypted
    )
    exit /b 1
)

echo.
echo ============================================
echo EXTRACTING TO TEMP DIRECTORY
echo ============================================

REM Create temp directory
set "TEMP_DIR=%TEMP%\ziplock_debug_%RANDOM%"
mkdir "%TEMP_DIR%"
echo Temp directory: %TEMP_DIR%

REM Extract archive
if "%PASSWORD%"=="" (
    %SEVENZ% x "%ARCHIVE_PATH%" -o"%TEMP_DIR%" -y
) else (
    %SEVENZ% x "%ARCHIVE_PATH%" -o"%TEMP_DIR%" -y -p%PASSWORD%
)

if %errorlevel% neq 0 (
    echo ERROR: Failed to extract archive
    rmdir /s /q "%TEMP_DIR%" 2>nul
    exit /b 1
)

echo Extraction successful

echo.
echo ============================================
echo EXAMINING EXTRACTED FILES
echo ============================================

REM Show directory tree
echo Directory structure:
dir "%TEMP_DIR%" /s /b

echo.
echo ============================================
echo METADATA EXAMINATION
echo ============================================

if exist "%TEMP_DIR%\metadata.yml" (
    for %%A in ("%TEMP_DIR%\metadata.yml") do set "META_SIZE=%%~zA"
    echo Found metadata.yml (!META_SIZE! bytes)
    echo.
    echo Metadata content:
    echo ----------------------------------------
    type "%TEMP_DIR%\metadata.yml"
    echo ----------------------------------------

    REM Look for credential_count
    findstr /i "credential_count" "%TEMP_DIR%\metadata.yml" >nul
    if !errorlevel! equ 0 (
        echo.
        echo Credential count from metadata:
        findstr /i "credential_count" "%TEMP_DIR%\metadata.yml"
    )
) else (
    echo ERROR: metadata.yml not found in archive
    goto cleanup
)

echo.
echo ============================================
echo CREDENTIAL EXAMINATION
echo ============================================

if exist "%TEMP_DIR%\credentials" (
    echo Found credentials directory

    REM Count credential folders
    set "CRED_COUNT=0"
    for /d %%D in ("%TEMP_DIR%\credentials\*") do (
        set /a CRED_COUNT+=1
        echo Found credential folder: %%~nxD

        REM Check for record.yml in each folder
        if exist "%%D\record.yml" (
            for %%F in ("%%D\record.yml") do set "RECORD_SIZE=%%~zF"
            echo   - record.yml found (!RECORD_SIZE! bytes)

            REM Show first few lines of record
            echo   - Content preview:
            more +0 "%%D\record.yml" | findstr /n "^" | more +1
        ) else (
            echo   - ERROR: record.yml missing in %%~nxD
        )
        echo.
    )

    echo Total credential folders found: !CRED_COUNT!
) else (
    echo ERROR: credentials directory not found in archive
    set "CRED_COUNT=0"
)

echo.
echo ============================================
echo DIAGNOSIS
echo ============================================

REM Extract credential_count from metadata
set "META_COUNT="
for /f "tokens=2 delims=: " %%A in ('findstr /i "credential_count" "%TEMP_DIR%\metadata.yml" 2^>nul') do (
    set "META_COUNT=%%A"
)

if "%META_COUNT%"=="" (
    echo Could not extract credential_count from metadata
) else (
    echo Expected credentials (from metadata): %META_COUNT%
    echo Actual credentials found: %CRED_COUNT%

    if %META_COUNT% equ %CRED_COUNT% (
        echo SUCCESS: Credential count matches - no mismatch detected
        echo The issue might be in the ZipLock loading logic, not the archive format
    ) else (
        echo ERROR: MISMATCH DETECTED - Expected %META_COUNT%, found %CRED_COUNT%

        if %CRED_COUNT% equ 0 (
            if %META_COUNT% gtr 0 (
                echo This is the exact issue reported: metadata claims credentials exist but none are found
                echo.
                echo Possible causes:
                echo 1. Credential files were not written during archive creation
                echo 2. Credential files were written with wrong paths/names
                echo 3. 7z compression failed to include credential files
                echo 4. Windows path handling issues during file creation
            )
        )
    )
)

echo.
echo ============================================
echo RECOMMENDATIONS
echo ============================================

if defined META_COUNT (
    if %CRED_COUNT% lss %META_COUNT% (
        echo DEBUGGING STEPS:
        echo 1. Add detailed logging to DesktopFileProvider::create_archive()
        echo 2. Verify temp directory contents before 7z compression
        echo 3. Check if credential files are actually written to temp dir
        echo 4. Test with simpler paths (no nested directories)
        echo 5. Compare Linux vs Windows temp directory behavior
        echo.
        echo POTENTIAL FIXES:
        echo 1. Use absolute paths when creating temp files
        echo 2. Add Windows-specific path normalization
        echo 3. Verify temp file permissions on Windows
        echo 4. Add file existence checks after each write operation
        echo 5. Use a flatter directory structure for Windows
    ) else (
        echo Archive structure appears correct
        echo The issue might be in the ZipLock loading/parsing logic
    )
)

:cleanup
echo.
echo Cleaning up temp directory...
rmdir /s /q "%TEMP_DIR%" 2>nul

echo.
echo Diagnostic complete.
