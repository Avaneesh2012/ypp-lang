# Y++ Programming Language (v1.0)

Y++ is a modern, simple, and flexible programming language built for fast execution, block-scoped namespaces, built-in socket networking, and interactive terminal applications. It comes bundled with a dark-mode graphical IDE, syntax highlighter, and built-in autocomplete.

---

## Installation & Setup

Installing Y++ takes just one click. Make sure Java 17 or higher is installed on your computer.

### Windows
1. Download or clone this repository.
2. Double-click `install-windows.bat`.
3. Launch the IDE anytime by double-clicking `ypp-ide.bat`.

### Mac and Linux
1. Download or clone this repository.
2. Open terminal and run:
   ```bash
   chmod +x install-mac-linux.sh && ./install-mac-linux.sh
   ```
3. Launch the IDE by running `./ypp-ide`.

---

## Y++ Code Syntax & Examples

### 1. Imports & Includes
```ypp
Import ycomponents *
Import ynetworking *
```

### 2. Printing & Type Casting
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
    integer applecount = 4i,
    smallint applesmall = 1si,
    double appleweight = 2.4d,
}

STRING 1 {
    slong fun = "fun",
    schar dumb = "d",
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

## Programming Examples Folder

All example Y++ programs, socket networking scripts, client-server templates, and syntax demonstrations are also saved inside the `programming examples` folder.

You can open the `programming examples` folder directly in the Y++ IDE using the **Open Folder** button in the sidebar.

---

## IDE Features

- **Built-in Graphical IDE**: Complete code editor with line numbers, custom themes, and project sidebar.
- **Auto-Complete & IntelliSense**: Suggestions for keywords, functions, and networking components.
- **Interactive Terminal**: Built-in console bar that lights up when input is required.
- **Debug Mode**: One-click visual execution tracer.

---

## License & Author

Created by Avaneesh (2026).
Version 1.0.0