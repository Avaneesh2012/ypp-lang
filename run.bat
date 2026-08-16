@echo off
REM Y++ build and run script
REM Usage:
REM   run.bat              -> interactive shell
REM   run.bat file.ypp     -> run a .ypp file
REM   run.bat -e "code"    -> run inline code

REM ── Try to find javac/java on PATH first ───────────────────────────
where javac >nul 2>&1
if %errorlevel% == 0 (
    set JAVAC=javac
    set JAVA=java
    goto compile
)

REM ── Fall back to IntelliJ bundled JDK ─────────────────────────────
set JBR=C:\Program Files\JetBrains\IntelliJ IDEA 2023.3.6\jbr\bin
if exist "%JBR%\javac.exe" (
    set JAVAC="%JBR%\javac.exe"
    set JAVA="%JBR%\java.exe"
    goto compile
)

echo ERROR: Java not found. Please install a JDK or add it to PATH.
exit /b 1

:compile
if not exist out mkdir out
echo Compiling Y++...
%JAVAC% -encoding UTF-8 -d out src\*.java
if %errorlevel% neq 0 (
    echo Compilation failed.
    exit /b 1
)
echo Done.
echo.

%JAVA% -cp out ypp.Ypp %*
