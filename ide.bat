@echo off
title Y++ IDE

REM ── Locate Java ───────────────────────────────────────────────────
where javac >nul 2>&1
if %errorlevel% == 0 (
    set JAVAC=javac
    set JAVA=java
    goto compile
)

set JBR=C:\Program Files\JetBrains\IntelliJ IDEA 2023.3.6\jbr\bin
if exist "%JBR%\javac.exe" (
    set JAVAC="%JBR%\javac.exe"
    set JAVA="%JBR%\java.exe"
    goto compile
)

echo [ERROR] Java not found.
pause & exit /b 1

:compile
if not exist out mkdir out
%JAVAC% -encoding UTF-8 -d out src\*.java
if %errorlevel% neq 0 ( echo Compilation failed. & pause & exit /b 1 )

:run
start "" %JAVA% -cp out ypp.YppIDE
