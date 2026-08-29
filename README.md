# Y++ Programming Language [v1.0.0 Stable]

Y++ is an interpreted, block-scoped programming language designed for clean syntax, explicit type bounds, built-in TCP socket networking, and seamless interactive terminal applications. 

Y++ is now a **native executable**. It runs instantly with **zero dependencies** — no JVM or runtime required!

```ypp
Import ycomponents *

PRINT: "Hello, World!";
```

---

## Why Y++?

Most programming languages either require heavy toolchains or force verbose boilerplate for basic tasks like networking and variable scope isolation. Y++ solves this with a developer-first approach:

1. **Zero-Friction Interpreter**: Download a single executable and run your code instantly. No Java, Python, or Node required.
2. **Explicit Block Namespaces**: `NUM` and `STRING` blocks isolate data memory and enforce exact numerical & string bounds (`smallint`, `integer`, `double`, `slong`, `schar`).
3. **Built-in Socket Networking (`ynetworking`)**: Create TCP servers and clients in under 10 lines of code with high-level stream wrappers.
4. **Human-Centric Error Diagnostics**: Clear, actionable error messages pinpointing the exact line and issue instead of vague crashes.
5. **Interactive REPL**: A built-in command-line shell to test code interactively.

---

## Installation & Quick Start

Installing Y++ is incredibly easy. Just download the single binary for your operating system!

### Download pre-built binaries (Recommended)
1. Go to the [Releases](../../releases) page.
2. Download the binary for your OS:
   - **Windows**: `ypp-windows-amd64.exe`
   - **Mac**: `ypp-macos-amd64`
   - **Linux**: `ypp-linux-amd64`
3. Open your terminal and run it!
   ```bash
   ./ypp examples/hello.ypp
   ```

### Build from source
If you have Rust installed, you can compile it yourself:
```bash
git clone https://github.com/Avaneesh2012/ypp-lang.git
cd ypp-lang
cargo build --release
./target/release/ypp
```

---

## Developer Experience (DX) & Error Messaging

Y++ prioritizes helpful error diagnostics over cryptic stack traces:

- **Missing Semicolons**:
  `[Y++ Error] Line 14: missing ';' after name.input(...) — semicolons are required on input calls.`
- **Strict Type Bounds**:
  `[Y++ Error] Line 21: schar parameter 'name' can only hold a single character, but got "avaneesh"`
- **Out of Range Numbers**:
  `[Y++ Error] Line 8: smallint value 5000 out of range [-1000, 1000]`

---

## Ready-to-Run Examples

Small, functional example programs are located in the `examples/` directory:

- `examples/hello.ypp` — Hello world and basic type casts
- `examples/example2.ypp` — Block namespaces and parameters
- `examples/networking.ypp` — TCP Client/Server socket communication template

---

## Language Syntax Reference

### 1. Imports & Includes
```ypp
Import ycomponents *
Import ynetworking *
```

### 2. Printing & Explicit Type Casts
```ypp
PRINT: "Hello, World!";

PRINT: string() textValue;
PRINT: int() numericValue;
PRINT: double() decimalValue;
PRINT: stringint() combinedValue;
```

### 3. NUM & STRING Block Namespaces
```ypp
NUM 1 {
    integer applecount = 4i,     // 64-bit Integer
    smallint applesmall = 1si,   // [-1000, 1000] Integer
    double appleweight = 2.4d,   // 15-digit Precision Double
}

STRING 1 {
    slong fun = "fun",           // Full Text String
    schar dumb = "d",           // Single Character String
}

together = (STRING 1)fun + " " + (STRING 1)dumb;
PRINT: string() together;
```

### 4. Functions & Instantiation Aliases
```ypp
func example() { 
    NUM 1 { 
        integer applecount = 4i;
    }
   
    NEW example.(NUM 1) = exampleclass;
    NEW example = exampleeverything;

    global NUM 2 {
        PRINT: "Avaneesh";
    }
}

exampleclass();
exampleeverything();

runexamples = new examples(\parameters\name);
runexamples();
```

### 5. Interactive Parameter Inputs
```ypp
func examples(slong name) { 
    NUM 1 { 
        slong.name = name; 
        name.input("What is your name: ");
        name.next();
        name.break;
    }
}
```

### 6. Socket Networking (Client & Server)

#### Server
```ypp
Import ycomponents *
Import ynetworking *

func server {
    server = new Server(5000);
    PRINT: "Server started. Waiting for client...";
    
    socket = server.accept();
    in = new primitivedataStream(socket.inputstream());
    
    STRING line;
    while (NOT: line = in.readutf-8().equals("End")) {
        PRINT: "Client says: " + line;
    }
    PRINT: "Client disconnected.";
}

server();
```

#### Client
```ypp
Import ycomponents *
Import ynetworking *

func client {
    networking = new Network("127.0.0.1", 5000);
    out = new primitivedataStream(networking.outstream());
    reader = reader(userinput());
    
    PRINT: "Connected to Server. Type your message:";
    STRING line;
    while (NOT: line = reader.readline().equals("End")) {
        out.utf-8(line);
    }
    out.utf-8("End");
}

client();
```

### 7. Control Flow & Loops
```ypp
STRING line;
while (NOT: line = reader.readline().equals("End")) {
    out.utf-8(line);
}
```

### 8. Exception Concat Blocks
```ypp
EXCEPTION CONCAT() {
    maybetogether = (STRING 1)fun + (NUM 1)applecount;
    PRINT: stringint() maybetogether;
}
```

---

## License & Author

Created by **Avaneesh** (2026).  
Version 1.0.0 Stable