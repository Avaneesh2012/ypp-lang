@echo off
title Y++ Shell
color 0A

REM ── Locate Java ───────────────────────────────────────────────────
where javac >nul 2>&1
if %errorlevel% == 0 (
    set JAVAC=javac
    set JAVA=java
    goto check_compile
)

set JBR=C:\Program Files\JetBrains\IntelliJ IDEA 2023.3.6\jbr\bin
if exist "%JBR%\javac.exe" (
    set JAVAC="%JBR%\javac.exe"
    set JAVA="%JBR%\java.exe"
    goto check_compile
)

echo  [ERROR] Java not found. Install a JDK and add it to PATH.
pause
exit /b 1

:check_compile
REM ── Only recompile if source is newer than class files ────────────
if not exist out\ypp\Ypp.class goto compile

REM Check if any .java source is newer than the compiled output
for %%f in (src\*.java) do (
    xcopy "%%f" out\ /d /y >nul 2>&1
    if errorlevel 1 goto compile
)
goto run

:compile
if not exist out mkdir out
echo  Compiling Y++...
%JAVAC% -encoding UTF-8 -d out src\*.java
if %errorlevel% neq 0 (
    echo  [ERROR] Compilation failed.
    pause
    exit /b 1
)
echo  Compiled OK.
echo.

:run
%JAVA% -cp out ypp.Ypp
