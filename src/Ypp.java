// Y++ Shell — Main entry point
package ypp;

import java.io.*;
import java.nio.file.*;
import java.util.*;

public class Ypp {

    static final String VERSION = "1.0.0";

    public static void main(String[] args) throws IOException {
        if (args.length == 0) {
            // Interactive REPL shell
            runShell();
        } else if (args.length == 2 && args[0].equals("-e")) {
            // Inline:  java ypp.Ypp -e "PRINT: \"hi\";"
            runCode(args[1], "<inline>");
        } else if (args.length == 1 && !args[0].startsWith("-")) {
            // File mode:  java ypp.Ypp myfile.ypp
            runFile(args[0]);
        } else {
            printHelp();
        }
    }

    // ------------------------------------------------------------------
    // File runner
    // ------------------------------------------------------------------

    private static void runFile(String path) throws IOException {
        File f = new File(path);
        if (!f.exists()) {
            System.err.println("Error: file not found — " + path);
            System.exit(1);
        }
        String source = Files.readString(f.toPath());
        runCode(source, path);
    }

    // ------------------------------------------------------------------
    // Core run: lex → parse → interpret
    // ------------------------------------------------------------------

    private static void runCode(String source, String origin) {
        try {
            List<Token>      tokens = new Lexer(source).tokenize();
            ASTNode.Program  ast    = new Parser(tokens).parse();
            new Interpreter().run(ast);
        } catch (YppException e) {
            System.err.println("[Y++ Error] " + e.getMessage());
        } catch (Exception e) {
            System.err.println("[Y++ Internal Error] " + e.getMessage());
        }
    }

    // ------------------------------------------------------------------
    // Interactive REPL shell
    // ------------------------------------------------------------------

    private static void runShell() throws IOException {
        printBanner();

        BufferedReader reader = new BufferedReader(new InputStreamReader(System.in));

        // The shell maintains ONE persistent interpreter across the session
        // so variables and NUM blocks survive between lines.
        Interpreter interp = new Interpreter();

        StringBuilder buffer      = new StringBuilder();
        int           braceDepth  = 0;    // track open { ... }
        boolean       inBlock     = false;

        System.out.print("ypp> ");
        System.out.flush();

        String line;
        while ((line = reader.readLine()) != null) {
            String trimmed = line.trim();

            // Built-in shell commands
            if (trimmed.equalsIgnoreCase("exit") || trimmed.equalsIgnoreCase("quit")) {
                System.out.println("Goodbye!");
                break;
            }
            if (trimmed.equalsIgnoreCase("help")) {
                printHelp();
                System.out.print("ypp> ");
                System.out.flush();
                continue;
            }
            if (trimmed.equalsIgnoreCase("clear")) {
                interp = new Interpreter();   // reset state
                System.out.println("-- environment cleared --");
                System.out.print("ypp> ");
                System.out.flush();
                continue;
            }

            // Append the line to the buffer
            buffer.append(line).append("\n");

            // Count brace depth to detect multi-line blocks
            for (char ch : line.toCharArray()) {
                if (ch == '{') { braceDepth++; inBlock = true; }
                if (ch == '}') { braceDepth--; }
            }

            boolean blockComplete = inBlock && braceDepth <= 0;
            boolean singleLine    = !inBlock;

            if (blockComplete || singleLine) {
                // Execute the buffered code
                String code = buffer.toString();
                buffer.setLength(0);
                braceDepth = 0;
                inBlock    = false;

                executeInShell(interp, code);

                System.out.print("ypp> ");
                System.out.flush();
            } else {
                // Still inside a block — show continuation prompt
                System.out.print("...  ");
                System.out.flush();
            }
        }
    }

    /**
     * Execute a snippet inside the shell.
     * We re-parse each time but share the same Interpreter instance,
     * so the environment (NUM blocks, globals) persists.
     */
    private static void executeInShell(Interpreter interp, String code) {
        try {
            List<Token>     tokens = new Lexer(code).tokenize();
            ASTNode.Program ast    = new Parser(tokens).parse();
            interp.run(ast);
        } catch (YppException e) {
            System.out.println("[Error] " + e.getMessage());
        } catch (Exception e) {
            System.out.println("[Error] " + e.getMessage());
        }
    }

    // ------------------------------------------------------------------
    // Help & banner
    // ------------------------------------------------------------------

    private static void printBanner() {
        System.out.println("+======================================+");
        System.out.println("|   Y++  Language Shell  v" + VERSION + "      |");
        System.out.println("|   Type 'exit' to quit, 'help' for   |");
        System.out.println("|   commands,  'clear' to reset env.  |");
        System.out.println("+======================================+");
        System.out.println();
    }

    private static void printHelp() {
        System.out.println();
        System.out.println("Usage:");
        System.out.println("  java -cp out ypp.Ypp                  → interactive shell");
        System.out.println("  java -cp out ypp.Ypp file.ypp         → run a .ypp file");
        System.out.println("  java -cp out ypp.Ypp -e \"<code>\"      → run inline code");
        System.out.println();
        System.out.println("Shell commands:");
        System.out.println("  exit / quit   exit the shell");
        System.out.println("  clear         reset the environment (clears all variables)");
        System.out.println("  help          show this message");
        System.out.println();
        System.out.println("Y++ quick reference:");
        System.out.println("  PRINT: \"Hello\";                  print a string");
        System.out.println("  PRINT: int() varname;            print rounded integer");
        System.out.println("  PRINT: double() varname;         print double value");
        System.out.println("  NUM 1 { integer x = 3i, }        declare a NUM block");
        System.out.println("  together = (NUM 1)x * (NUM 1)y;  arithmetic with blocks");
        System.out.println("  \\\\this is a comment\\\\            block comment");
        System.out.println();
    }
}
