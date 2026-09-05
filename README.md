# Y++ Programming Language [v1.5.0]

Y++ is an interpreted, block-scoped programming language designed for clean syntax, explicit type bounds, built-in TCP socket networking, GUI/game windows, and interactive terminal applications.

Y++ is a **native executable**. Download a binary for your OS — no JVM or Node required.

```ypp
Import ycomponents *

PRINT: "Hello, World!";
```

---

## Why Y++?

1. **Zero-Friction Interpreter**: Download a single executable and run your code instantly.
2. **Explicit Block Namespaces**: `NUM` and `STRING` blocks isolate data memory and enforce exact numerical & string bounds (`smallint`, `integer`, `double`, `slong`, `schar`).
3. **Built-in Socket Networking (`ynetworking`)**: TCP servers and clients with stream wrappers.
4. **Built-in GUI & Games (`yGUI`)**: Open a window, draw entities, and poll `KEYBOARD.KEYHOLD` — and mix it with `ynetworking` in the same program.
5. **Human-Centric Error Diagnostics**: Clear messages with line numbers.
6. **Interactive REPL**: A built-in command-line shell.

---

## Installation & Quick Start

### Download pre-built binaries (Recommended)
1. Go to the [Releases](https://github.com/Avaneesh2012/ypp-lang/releases) page.
2. Download the binary for your OS:
   - **Windows**: `ypp-windows-amd64.exe`
   - **Mac**: `ypp-macos-amd64`
   - **Linux**: `ypp-linux-amd64`
3. Open your terminal and run it:
   ```bash
   ./ypp examples/hello.ypp
   ```

### Build from source
```bash
git clone https://github.com/Avaneesh2012/ypp-lang.git
cd ypp-lang
cargo build --release
./target/release/ypp
```

---

## Developer Experience (DX) & Error Messaging

- **Missing Semicolons**:
  `[Y++ Error] Line 14: missing ';' after name.input(...) — semicolons are required on input calls.`
- **Strict Type Bounds**:
  `[Y++ Error] Line 21: schar parameter 'name' can only hold a single character, but got "avaneesh"`
- **Out of Range Numbers**:
  `[Y++ Error] Line 8: smallint value 5000 out of range [-1000, 1000]`
- **Missing packages**:
  `[Y++ Error] Line 10: Import yGUI is required to use KEYBOARD.`

---

## Ready-to-Run Examples

- `examples/hello.ypp` — Hello world and basic type casts
- `examples/example2.ypp` — Block namespaces and parameters
- `examples/networking.ypp` — TCP client/server template
- `examples/example6.ypp` — GUI window, entity, WASD movement (`Import yGUI *`)

---

## Language Syntax Reference

### 1. Imports & Includes

Imports only enable **built-in** packages. They never load files from disk.

```ypp
Import ycomponents *
Import ynetworking *
Import yGUI *
```

You can combine packages in one program (for example a networked game: `ycomponents` + `ynetworking` + `yGUI`).

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

Requires `Import ynetworking *`. Connections use timeouts and a 64KiB line limit.

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

### 7. GUI & Games (`yGUI`)

Requires `Import yGUI *`. Frame sizes are a **cell grid** (not raw pixels). Each cell is 10 pixels, and sizes are clamped to 256×256 cells.

Colors: `BLACK`, `WHITE`, `RED`, `GREEN`, `BLUE`, `YELLOW`, `CYAN`, `MAGENTA`, `ORANGE`, `GRAY`, `PINK`, and more.

```ypp
Import ycomponents *
Import yGUI *

NUM framedimensions {
    integer framewidth = 60;
    integer framelength = 70;
}

NEW yGUI.(NUM framedimensions) = setup;
frame = setup();

frame.dimensions(60, 70);
frame.color = BLACK;

func entity(width, length) {
    NUM entitydimensions {
        integer width = width;
        integer length = length;
    }
}

NEW entity.(NUM entitydimensions) = setup;
setup();

playerwidth = entity.width;
playerlength = entity.length;
player = dimensions(playerlength, playerwidth);

func entityMovement(x, y) {
    STRING controls {
        schar w,a,s,d = ['w','a','s','d'];
    }

    if controls.w == KEYBOARD.KEYHOLD('w') {
        y++;
    }
    if controls.a == KEYBOARD.KEYHOLD('a') {
        x--;
    }
    if controls.s == KEYBOARD.KEYHOLD('s') {
        y--;
    }
    if controls.d == KEYBOARD.KEYHOLD('d') {
        x++;
    }
}

NEW entityMovement = setup;
setup();

bool trueforever = TRUE;
while trueforever {
    yGUI();
    entity();
    entityMovement();
}
```

Close the window to end the loop. See `examples/example6.ypp`.

### 8. Control Flow & Loops
```ypp
if x == 1 {
    PRINT: "one";
}

STRING line;
while (NOT: line = reader.readline().equals("End")) {
    out.utf-8(line);
}
```

### 9. Exception Concat Blocks
```ypp
EXCEPTION CONCAT() {
    maybetogether = (STRING 1)fun + (NUM 1)applecount;
    PRINT: stringint() maybetogether;
}
```

---

## License & Author

Created by **Avaneesh** (2026).  
Version 1.5.0
