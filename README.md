# 🚀 Y++ Programming Language (v1.0)

Welcome to **Y++ (v1.0)**! Y++ is a flexible, block-scoped, and modern programming language designed for simplicity, expressive syntax, built-in networking, and interactive terminal applications. It comes bundled with its own dark-themed **IDE**, **syntax highlighter**, **IntelliSense auto-complete**, and **interactive terminal**.

---

## ⚡ Quick Start & Installation (3 Seconds)

Y++ runs on any operating system with Java 17+ installed. No complex setup or package managers required!

### 1. Download & Clone
```bash
git clone https://github.com/Avaneesh2012/ypp-lang.git
cd ypp-lang
```

### 2. Launch the Y++ IDE
- **Windows**: Double-click `ide.bat` or run:
  ```cmd
  ide.bat
  ```
- **Mac / Linux**: Run:
  ```bash
  javac -d out src/*.java && java -cp out ypp.YppIDE
  ```

### 3. Run Y++ Scripts from the Command Line
- **Interactive REPL Shell**:
  ```bash
  java -cp out ypp.Ypp
  ```
- **Run a Script File**:
  ```bash
  java -cp out ypp.Ypp hello.ypp
  ```

---

## 💡 Y++ Syntax Guide & Features

### 1. Packages & Includes
At the top of every Y++ script, import standard modules using:
```ypp
Import ycomponents *
Import ynetworking *
```

### 2. Printing & Casts
Output values directly to the terminal using `PRINT:`:
```ypp
PRINT: "Hello, World!";

// Explicit Type Casting
PRINT: string() together;
PRINT: int() numericVal;
PRINT: double() decimalVal;
PRINT: stringint() combinedVal;
```

---

### 3. NUM & STRING Block Namespaces
Variables in Y++ belong to labelled block namespaces (`NUM` for numbers, `STRING` for text):

```ypp
NUM 1 {
    integer applecount = 4i,     // 64-bit Integer (suffix 'i')
    smallint applesmall = 1si,   // Small Integer [-1000, 1000] (suffix 'si')
    double appleweight = 2.4d,   // Double precision decimal (suffix 'd')
}

STRING 1 {
    slong fun = "fun",           // Full text string
    schar dumb = "d",           // Single character string
}

// Block Variable Access: (NUM label)varName  or  (STRING label)varName
together = (STRING 1)fun + " " + (STRING 1)dumb;
PRINT: string() together;
```

---

### 4. Functions & `NEW` Aliases
Functions declare reusable scopes. You can create global aliases to functions or specific blocks inside functions:

```ypp
func example() { 
    NUM 1 { 
        integer applecount = 4i;
    }
   
    // Create alias to specific block
    NEW example.(NUM 1) = exampleclass;

    // Create alias to whole function
    NEW example = exampleeverything;

    // Auto-executing global block
    global NUM 2 {
        PRINT: "Avaneesh";
    }
}

// Execute aliases
exampleclass();
exampleeverything();
```

Alternatively, instantiate aliases using the `new` operator:
```ypp
runexamples = new examples(\parameters\name);
runspecificexamples = new (NUM 1).examples(\parameters\name);

runexamples();
runspecificexamples();
```

---

### 5. Interactive Parameter Inputs
Prompt users for terminal input using parameter methods:

```ypp
func examples(slong name) { 
    NUM 1 { 
        slong.name = name; 
        name.input("What is your name: ");   // Pauses terminal for user input
        name.next();                        // Newline
        name.break;                         // Stop questionnaire
    }
}
```

> **Note**: Type checking is strictly enforced! Attempting to enter a string longer than 1 character into a `schar` parameter will produce a clean Y++ runtime error.

---

### 6. 🌐 Socket Networking (`ynetworking`)

Y++ features built-in socket networking out of the box!

#### Server Socket (`server.ypp`)
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

#### Client Socket (`client.ypp`)
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

---

### 7. Control Flow & Loops
Y++ supports `while` loops, inline assignment, and logical inversion (`NOT:` or `!`):

```ypp
STRING line;
while (NOT: line = reader.readline().equals("End")) {
    out.utf-8(line);
}
```

---

### 8. Exception Concat Blocks
Catch and handle string & numeric exceptions safely:

```ypp
EXCEPTION CONCAT() {
    maybetogether = (STRING 1)fun + (NUM 1)applecount;
    PRINT: stringint() maybetogether;
}
```

---

## 🎨 Y++ IDE Features

- **Dark Mode Modern Aesthetic**: Clean sidebar, line numbers, status bar, and toolbar.
- **Project Explorer Sidebar**: Right-click to create **New File**, **New Folder**, or **New Package** (automatically adds `Import <package> *` to your file).
- **IntelliSense / Autocomplete**: Press `Tab` or `Enter` for pop-up keyword suggestions.
- **Interactive Terminal Bar**: Bottom console lights up green when `name.input(...)` or `reader.readline()` requests user input.
- **Debug Mode Button**: Click **Debug** to view live AST execution traces in real time.

---

## 📜 License & Author

Created with ❤️ by **Avaneesh** (2026).
Feel free to star ⭐️ the repository and contribute!