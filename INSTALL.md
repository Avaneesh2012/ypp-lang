# Installation Guide for Y++ Compiler

## Step 1: Install Julia

### Windows

1. Download Julia from [https://julialang.org/downloads/](https://julialang.org/downloads/)
2. Run the installer (e.g., `julia-1.x.x-win64.exe`)
3. During installation, check the option "Add Julia to PATH"
4. Complete the installation

### Alternative: Manual PATH Setup (Windows)

If you didn't add Julia to PATH during installation:

1. Find your Julia installation directory (usually `C:\Users\YourUsername\AppData\Local\Programs\Julia-x.x.x\bin`)
2. Open System Properties → Advanced → Environment Variables
3. Under "System variables", find and edit "Path"
4. Click "New" and add the Julia bin directory path
5. Click OK to save

### macOS

```bash
# Using Homebrew
brew install julia

# Or download from julialang.org and drag to Applications
```

### Linux

```bash
# Ubuntu/Debian
sudo apt-get install julia

# Fedora
sudo dnf install julia

# Or download from julialang.org
```

## Step 2: Verify Installation

Open a new terminal/command prompt and run:

```bash
julia --version
```

You should see output like: `julia version 1.x.x`

## Step 3: Test Y++ Compiler

Navigate to the Y++ project directory:

```bash
cd "d:/ypp programming language"
```

Run the test suite:

```bash
julia test.jl
```

Run an example program:

```bash
julia ypp.jl example.ypp
```

## Troubleshooting

### "julia is not recognized" error

- Make sure Julia is added to your PATH
- Restart your terminal/command prompt after installation
- Try using the full path to julia.exe:
  ```bash
  "C:\Users\YourUsername\AppData\Local\Programs\Julia-1.x.x\bin\julia.exe" ypp.jl example.ypp
  ```

### Permission errors

- On Windows, run terminal as Administrator
- On macOS/Linux, use `sudo` if needed

### File not found errors

- Make sure you're in the correct directory
- Use `cd` to navigate to the project folder
- Check that all files exist using `dir` (Windows) or `ls` (macOS/Linux)

## Quick Start After Installation

1. Create a new Y++ file (e.g., `myprogram.ypp`)
2. Write your Y++ code
3. Run it: `julia ypp.jl myprogram.ypp`

## Example Commands

```bash
# Run a Y++ file
julia ypp.jl example.ypp

# Run Y++ code directly
julia ypp.jl -e "nvar x = 10; PRINT: x;"

# Run tests
julia test.jl

# Run simple test
julia ypp.jl test_simple.ypp
```

## Next Steps

- Read `README.md` for language syntax and features
- Explore `example.ypp` for code examples
- Create your own Y++ programs!