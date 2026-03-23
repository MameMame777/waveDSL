@echo off
setlocal enabledelayedexpansion

:: ---------------------------------------------------------------------------
:: make-release.bat
:: Generates three release artifacts:
::   wavedsl-vX.Y.Z-windows-x86_64.zip  -- pre-built binary
::   wavedsl-vX.Y.Z-src-vendored.tar.gz -- source + all deps vendored
::   CHECKSUMS.txt                       -- SHA-256 of both
:: ---------------------------------------------------------------------------

:: Read version from Cargo.toml via PowerShell
for /f %%v in ('powershell -NoProfile -Command "(Select-String -Path Cargo.toml -Pattern '^version = ""(.+)""').Matches[0].Groups[1].Value"') do set VERSION=%%v

if "%VERSION%"=="" (
    echo ERROR: Could not read version from Cargo.toml
    exit /b 1
)

echo WaveDSL v%VERSION% release build
echo.

set BIN_ZIP=wavedsl-v%VERSION%-windows-x86_64.zip
set SRC_TGZ=wavedsl-v%VERSION%-src-vendored.tar.gz

:: --- Step 1: Build binary ---------------------------------------------------
echo [1/6] Building binary (cargo build --release --locked)...
cargo build --release --locked
if errorlevel 1 (
    echo ERROR: cargo build failed
    exit /b 1
)

:: --- Step 2: Binary zip -----------------------------------------------------
echo [2/6] Creating %BIN_ZIP%...
if exist "%BIN_ZIP%" del "%BIN_ZIP%"
powershell -NoProfile -Command ^
    "Compress-Archive -Path 'target\release\wavedsl.exe','README.md','LICENSE' -DestinationPath '%BIN_ZIP%' -Force"
if errorlevel 1 (
    echo ERROR: Compress-Archive failed
    exit /b 1
)

:: --- Step 3: Vendor dependencies --------------------------------------------
echo [3/6] Running cargo vendor...
if exist vendor rmdir /s /q vendor
cargo vendor vendor
if errorlevel 1 (
    echo ERROR: cargo vendor failed
    exit /b 1
)

:: --- Step 4: Write .cargo/config.toml (for the archive only) ---------------
echo [4/6] Writing .cargo\config.toml...
if not exist .cargo mkdir .cargo
(
echo [source.crates-io]
echo replace-with = "vendored-sources"
echo.
echo [source.vendored-sources]
echo directory = "vendor"
) > .cargo\config.toml

:: --- Step 5: Create vendored source archive ---------------------------------
:: Write to parent dir first to avoid "file changed as we read it" warning
:: (tar exit code 1 occurs when archiving a directory into itself)
echo [5/6] Creating %SRC_TGZ%...
if exist "%SRC_TGZ%" del "%SRC_TGZ%"
if exist "..\%SRC_TGZ%" del "..\%SRC_TGZ%"
tar -czf "..\%SRC_TGZ%" ^
    --exclude=".git" ^
    --exclude="target" ^
    --exclude="*.zip" ^
    --exclude="*.tar.gz" ^
    --exclude="CHECKSUMS.txt" ^
    --exclude="output.json" ^
    --exclude="axi_output.json" ^
    --exclude="burst_output.json" ^
    --exclude="testtest.json" ^
    .
if errorlevel 1 (
    echo ERROR: tar failed
    goto :cleanup
)
move "..\%SRC_TGZ%" "." >nul

:: --- Step 6: SHA-256 checksums ----------------------------------------------
echo [6/6] Generating CHECKSUMS.txt...
if exist CHECKSUMS.txt del CHECKSUMS.txt

for /f "skip=1 tokens=1" %%h in ('certutil -hashfile "%BIN_ZIP%" SHA256') do (
    if not defined HASH_BIN set HASH_BIN=%%h
)
for /f "skip=1 tokens=1" %%h in ('certutil -hashfile "%SRC_TGZ%" SHA256') do (
    if not defined HASH_SRC set HASH_SRC=%%h
)

(
    echo %HASH_BIN%  %BIN_ZIP%
    echo %HASH_SRC%  %SRC_TGZ%
) > CHECKSUMS.txt

:cleanup
:: Remove temporary files from the working tree
echo Cleaning up temporary files...
if exist vendor rmdir /s /q vendor
if exist .cargo\config.toml del .cargo\config.toml

echo.
echo Done. Upload these files to the GitHub Release:
echo   %BIN_ZIP%
echo   %SRC_TGZ%
echo   CHECKSUMS.txt
echo.
type CHECKSUMS.txt
