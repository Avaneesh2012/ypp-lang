// Y++ Interpreter — walks the AST and executes it
package ypp;

import java.util.*;
import java.util.concurrent.SynchronousQueue;

public class Interpreter {

    // ------------------------------------------------------------------
    // Input Provider — lets the IDE wire in an interactive terminal
    // ------------------------------------------------------------------

    public interface InputProvider {
        /** Print the prompt, then block until the user enters a line. */
        String readLine(String prompt);
    }

    /** Default: use System.in (fallback for headless/test runs) */
    public InputProvider inputProvider = prompt -> {
        System.out.print(prompt);
        try {
            java.io.BufferedReader br =
                new java.io.BufferedReader(new java.io.InputStreamReader(System.in));
            return br.readLine();
        } catch (java.io.IOException e) {
            return "";
        }
    };

    // ------------------------------------------------------------------
    // Runtime state
    // ------------------------------------------------------------------

    /** NUM block variables:  label → (varName → value) */
    private final Map<String, Map<String, Double>> numBlocks    = new LinkedHashMap<>();

    /** STRING block variables: label → (varName → value) */
    private final Map<String, Map<String, String>> stringBlocks = new LinkedHashMap<>();

    /** Global variables (assigned outside any block, can be Double or String) */
    private final Map<String, Object>              globals      = new LinkedHashMap<>();

    /** Parameter types: paramName -> type (e.g. "schar", "slong", "integer", etc.) */
    private final Map<String, String>              paramTypes   = new LinkedHashMap<>();

    public boolean debugMode = false;

    // ------------------------------------------------------------------
    // Functions and Aliases
    // ------------------------------------------------------------------

    private final Map<String, ASTNode.FuncDeclNode> functions = new HashMap<>();
    
    private static class AliasTarget {
        final String funcName;
        final String blockLabel;
        AliasTarget(String f, String b) { funcName = f; blockLabel = b; }
    }
    private final Map<String, AliasTarget> aliases = new HashMap<>();

    // ------------------------------------------------------------------
    // Data-type bounds (inclusive)
    // ------------------------------------------------------------------

    private static final long   SMALLINT_MIN = -1_000L;
    private static final long   SMALLINT_MAX =  1_000L;
    private static final long   INTEGER_MIN  = -1_000_000_000_000_000L;
    private static final long   INTEGER_MAX  =  1_000_000_000_000_000L;
    private static final double DOUBLE_MIN   = -5_677_719_218.1092;
    private static final double DOUBLE_MAX   =  5_677_719_218.1092;
    private static final int    DOUBLE_SCALE = 15;   // max decimal digits

    private final Set<String> importedPackages = new HashSet<>();

    // ------------------------------------------------------------------
    // Public API
    // ------------------------------------------------------------------

    public void run(ASTNode.Program program) {
        for (ASTNode stmt : program.statements) {
            execStatement(stmt);
        }
    }

    // ------------------------------------------------------------------
    // Statement execution
    // ------------------------------------------------------------------

    private void execStatement(ASTNode node) {
        if (debugMode) {
            System.out.println("[DEBUG] Executing: " + node.getClass().getSimpleName());
        }

        if (node instanceof ASTNode.ImportNode n) {
            execImport(n);
            return;
        }

        // Must import ycomponents before executing any other statement
        if (!importedPackages.contains("ycomponents")) {
            throw new YppException("Import ycomponents is required to run Y++ code.");
        }

        if (node instanceof ASTNode.NumBlockNode n) {
            execNumBlock(n);
        } else if (node instanceof ASTNode.StringBlockNode n) {
            execStringBlock(n);
        } else if (node instanceof ASTNode.ExceptionConcatNode n) {
            boolean oldFlag = inExceptionConcat;
            inExceptionConcat = true;
            try {
                for (ASTNode stmt : n.statements) {
                    execStatement(stmt);
                }
            } finally {
                inExceptionConcat = oldFlag;
            }
        } else if (node instanceof ASTNode.PrintNode n) {
            execPrint(n);
        } else if (node instanceof ASTNode.AssignNode n) {
            execAssign(n);
        } else if (node instanceof ASTNode.FuncDeclNode n) {
            if (functions.containsKey(n.name)) throw new YppException("Function already defined: " + n.name);
            functions.put(n.name, n);
            for (ASTNode.Param p : n.params) {
                paramTypes.put(p.name, p.type);
            }
            // Execute global blocks and NEW aliases immediately upon declaration
            for (ASTNode stmt : n.body) {
                registerGlobalsAndAliases(stmt);
            }
        } else if (node instanceof ASTNode.NewAliasNode n) {
            aliases.put(n.aliasName, new AliasTarget(n.funcName, n.blockLabel));
        } else if (node instanceof ASTNode.FuncCallNode n) {
            execFuncCall(n);
        } else if (node instanceof ASTNode.GlobalBlockNode n) {
            execStatement(n.innerBlock);
        } else if (node instanceof ASTNode.ParamBindNode n) {
            globals.putIfAbsent(n.paramName, "");
            paramTypes.put(n.paramName, n.type);
        } else if (node instanceof ASTNode.ParamInputNode n) {
            String prompt = String.valueOf(evalExprRaw(n.prompt));
            String result = inputProvider.readLine(prompt);
            if (result == null) result = "";
            System.out.println(result);   // echo input to terminal

            String pType = paramTypes.get(n.paramName);
            if (pType != null) {
                if ("schar".equals(pType) && result.length() > 1) {
                    throw new YppException("Line " + n.line + ": schar parameter '" + n.paramName + "' can only hold a single character, but got \"" + result + "\"");
                }
                if ("smallint".equals(pType) || "integer".equals(pType) || "double".equals(pType)) {
                    try {
                        double d = Double.parseDouble(result);
                        validateAndCoerce(pType, d, n.line);
                    } catch (NumberFormatException ex) {
                        throw new YppException("Line " + n.line + ": " + pType + " parameter '" + n.paramName + "' expected a number, but got \"" + result + "\"");
                    }
                }
            }

            globals.put(n.paramName, result);
        } else if (node instanceof ASTNode.ParamNextNode n) {
            System.out.println();
        } else if (node instanceof ASTNode.ParamBreakNode n) {
            // Break handled as no-op or UI flow stop
        } else if (node instanceof ASTNode.VarDeclNode n) {
            execVarDecl(n);
        } else if (node instanceof ASTNode.StringVarDeclNode n) {
            execStringVarDecl(n);
        } else if (node instanceof ASTNode.WhileNode n) {
            execWhile(n);
        } else if (node instanceof ASTNode.ExprNode n) {
            evalExprRaw(n);
        }
        // other node types are silently ignored at statement level
    }

    private void registerGlobalsAndAliases(ASTNode node) {
        if (node instanceof ASTNode.GlobalBlockNode g) {
            execStatement(g.innerBlock);
        } else if (node instanceof ASTNode.NewAliasNode a) {
            execStatement(a);
        } else if (node instanceof ASTNode.FuncDeclNode f) {
            execStatement(f);
        } else if (node instanceof ASTNode.NumBlockNode nb) {
            for (ASTNode s : nb.statements) registerGlobalsAndAliases(s);
        } else if (node instanceof ASTNode.StringBlockNode sb) {
            for (ASTNode s : sb.statements) registerGlobalsAndAliases(s);
        }
    }

    private void execFuncCall(ASTNode.FuncCallNode n) {
        if (debugMode) System.out.println("[DEBUG] Calling function/alias: " + n.aliasName);
        if (aliases.containsKey(n.aliasName)) {
            AliasTarget target = aliases.get(n.aliasName);
            ASTNode.FuncDeclNode func = functions.get(target.funcName);
            if (func == null) throw new YppException("Undefined function: " + target.funcName);
            
            for (ASTNode stmt : func.body) {
                if (target.blockLabel == null) {
                    execStatement(stmt);
                } else {
                    if (stmt instanceof ASTNode.NumBlockNode nb && nb.label != null && nb.label.equals(target.blockLabel)) {
                        execStatement(stmt);
                    }
                    if (stmt instanceof ASTNode.StringBlockNode sb && sb.label != null && sb.label.equals(target.blockLabel)) {
                        execStatement(stmt);
                    }
                }
            }
        } else if (functions.containsKey(n.aliasName)) {
            ASTNode.FuncDeclNode func = functions.get(n.aliasName);
            for (ASTNode stmt : func.body) execStatement(stmt);
        } else {
            throw new YppException("Line " + n.line + ": Undefined function or alias '" + n.aliasName + "'");
        }
    }

    private boolean inExceptionConcat = false;
    private String currentNumBlock = null;
    private String currentStringBlock = null;

    // ---- Import -------------------------------------------------------

    private void execImport(ASTNode.ImportNode n) {
        importedPackages.add(n.pkg);
    }

    // ---- Var Decls ----------------------------------------------------

    private void execVarDecl(ASTNode.VarDeclNode decl) {
        if (currentNumBlock == null) {
            System.err.println("[Y++ Error] Variable '" + decl.varName + "' declared outside a NUM block. Wrap it in: NUM <label> { ... }");
            return;
        }
        Map<String, Double> blockVars = numBlocks.computeIfAbsent(currentNumBlock, k -> new LinkedHashMap<>());
        Object valObj = evalExprRaw(decl.value);
        double value = (valObj instanceof Number num) ? num.doubleValue() : 0.0;
        value = validateAndCoerce(decl.typeName, value, decl.line);
        // Overwrite allowed — same variable in the same block gets updated
        blockVars.put(decl.varName, value);
        if (debugMode) System.out.println("[DEBUG] NUM[" + currentNumBlock + "]." + decl.varName + " = " + value);
    }

    private void execStringVarDecl(ASTNode.StringVarDeclNode decl) {
        if (currentStringBlock == null) {
            System.err.println("[Y++ Error] String variable '" + decl.varName + "' declared outside a STRING block. Wrap it in: STRING <label> { ... }");
            return;
        }
        Map<String, String> blockVars = stringBlocks.computeIfAbsent(currentStringBlock, k -> new LinkedHashMap<>());
        Object valObj = evalExprRaw(decl.value);
        String strVal = String.valueOf(valObj);
        if ("schar".equals(decl.typeName) && strVal.length() > 1) {
            throw new YppException("Line " + decl.line + ": schar '" + decl.varName + "' can only hold a single character, but got \"" + strVal + "\"");
        }
        // Overwrite allowed — same variable gets updated
        blockVars.put(decl.varName, strVal);
        if (debugMode) System.out.println("[DEBUG] STRING[" + currentStringBlock + "]." + decl.varName + " = " + strVal);
    }

    // ---- NUM block ----------------------------------------------------

    private void execNumBlock(ASTNode.NumBlockNode n) {
        if (n.label == null) {
            for (ASTNode stmt : n.statements) execStatement(stmt);
            return;
        }
        String oldNum = currentNumBlock;
        currentNumBlock = n.label;
        numBlocks.putIfAbsent(n.label, new LinkedHashMap<>());
        try {
            for (ASTNode stmt : n.statements) execStatement(stmt);
        } finally {
            currentNumBlock = oldNum;
        }
    }

    // ---- STRING block -------------------------------------------------

    private void execStringBlock(ASTNode.StringBlockNode n) {
        if (n.label == null) {
            for (ASTNode stmt : n.statements) execStatement(stmt);
            return;
        }
        String oldStr = currentStringBlock;
        currentStringBlock = n.label;
        stringBlocks.putIfAbsent(n.label, new LinkedHashMap<>());
        try {
            for (ASTNode stmt : n.statements) execStatement(stmt);
        } finally {
            currentStringBlock = oldStr;
        }
    }

    // ---- PRINT --------------------------------------------------------

    private void execPrint(ASTNode.PrintNode n) {
        Object val = evalExprRaw(n.expr);

        switch (n.cast) {
            case STRING, STRINGINT -> {
                System.out.println(String.valueOf(val));
            }
            case INT -> {
                if (val instanceof Number num) {
                    long rounded = Math.round(num.doubleValue());
                    System.out.println(rounded);
                } else {
                    try {
                        double d = Double.parseDouble(String.valueOf(val));
                        System.out.println(Math.round(d));
                    } catch (NumberFormatException e) {
                        System.out.println(val);
                    }
                }
            }
            case DOUBLE -> {
                if (val instanceof Number num) {
                    System.out.println(formatDouble(num.doubleValue()));
                } else {
                    try {
                        double d = Double.parseDouble(String.valueOf(val));
                        System.out.println(formatDouble(d));
                    } catch (NumberFormatException e) {
                        System.out.println(val);
                    }
                }
            }
            case NONE -> {
                if (val instanceof Double d) {
                    if (d == Math.floor(d) && !Double.isInfinite(d)) {
                        System.out.println((long) d.doubleValue());
                    } else {
                        System.out.println(formatDouble(d));
                    }
                } else {
                    System.out.println(String.valueOf(val));
                }
            }
        }
    }

    // ---- Assignment ---------------------------------------------------

    private void execAssign(ASTNode.AssignNode n) {
        Object value = evalExprRaw(n.value);
        if (debugMode) System.out.println("[DEBUG] Assigning " + n.name + " = " + value);
        globals.put(n.name, value);
    }

    // ------------------------------------------------------------------
    // Expression evaluation
    // ------------------------------------------------------------------

    private double evalExpr(ASTNode.ExprNode expr) {
        Object obj = evalExprRaw(expr);
        if (obj instanceof Number num) return num.doubleValue();
        throw new YppException("Expected numeric expression");
    }

    private Object evalExprRaw(ASTNode.ExprNode expr) {
        if (expr instanceof ASTNode.NumberLiteralNode n) {
            return n.value;
        }
        if (expr instanceof ASTNode.StringLiteralNode s) {
            return s.value;
        }
        if (expr instanceof ASTNode.IdentNode n) {
            if (!globals.containsKey(n.name)) {
                throw new YppException("Undefined variable: " + n.name);
            }
            return globals.get(n.name);
        }
        if (expr instanceof ASTNode.NumAccessNode n) {
            return resolveNumAccess(n);
        }
        if (expr instanceof ASTNode.StringAccessNode n) {
            return resolveStringAccess(n);
        }
        if (expr instanceof ASTNode.WhileNode n) {
            execWhile(n);
            return null;
        }
        if (expr instanceof ASTNode.NotExprNode n) {
            return evalNot(n);
        }
        if (expr instanceof ASTNode.InlineAssignExprNode n) {
            return evalInlineAssign(n);
        }
        if (expr instanceof ASTNode.NewObjectExprNode n) {
            return evalNewObject(n);
        }
        if (expr instanceof ASTNode.MethodCallExprNode n) {
            return evalMethodCall(n);
        }
        if (expr instanceof ASTNode.BinaryExprNode n) {
            Object left  = evalExprRaw(n.left);
            Object right = evalExprRaw(n.right);

            // Concatenation with '+' or mixed '*' in EXCEPTION CONCAT
            if (n.op == '+' || (n.op == '*' && inExceptionConcat && (left instanceof String || right instanceof String))) {
                boolean leftIsStr  = left instanceof String;
                boolean rightIsStr = right instanceof String;

                if (leftIsStr && rightIsStr) {
                    // String + String: direct concat; String * String: space-separated
                    return n.op == '+' ? (String.valueOf(left) + String.valueOf(right))
                                      : (String.valueOf(left) + " " + String.valueOf(right));
                }

                if (leftIsStr || rightIsStr) {
                    if (!inExceptionConcat) {
                        System.out.println("ERROR");
                        throw new YppException("Type error: string and number concatenation requires EXCEPTION CONCAT");
                    }
                    return formatVal(left) + " " + formatVal(right);
                }
            }

            double lNum = (left instanceof Number num) ? num.doubleValue() : 0.0;
            double rNum = (right instanceof Number num) ? num.doubleValue() : 0.0;

            return switch (n.op) {
                case '+' -> lNum + rNum;
                case '-' -> lNum - rNum;
                case '*' -> lNum * rNum;
                case '/' -> {
                    if (rNum == 0) throw new YppException("Division by zero");
                    yield lNum / rNum;
                }
                default -> throw new YppException("Unknown operator: " + n.op);
            };
        }
        throw new YppException("Unknown expression type: " + expr.getClass().getSimpleName());
    }

    private String formatVal(Object obj) {
        if (obj instanceof Double d) {
            if (d == Math.floor(d) && !Double.isInfinite(d)) {
                return String.valueOf((long) d.doubleValue());
            } else {
                return formatDouble(d);
            }
        }
        return String.valueOf(obj);
    }

    private double resolveNumAccess(ASTNode.NumAccessNode n) {
        Map<String, Double> block = numBlocks.get(n.blockLabel);
        if (block == null) {
            throw new YppException("Undefined NUM block: " + n.blockLabel);
        }
        if (!block.containsKey(n.varName)) {
            throw new YppException("Undefined variable '" + n.varName + "' in block " + n.blockLabel);
        }
        return block.get(n.varName);
    }

    private String resolveStringAccess(ASTNode.StringAccessNode n) {
        Map<String, String> block = stringBlocks.get(n.blockLabel);
        if (block == null) {
            throw new YppException("Undefined STRING block: " + n.blockLabel);
        }
        if (!block.containsKey(n.varName)) {
            throw new YppException("Undefined variable '" + n.varName + "' in block " + n.blockLabel);
        }
        return block.get(n.varName);
    }

    // ------------------------------------------------------------------
    // Type validation & coercion
    // ------------------------------------------------------------------

    private double validateAndCoerce(String typeName, double value, int line) {
        return switch (typeName) {
            case "smallint" -> {
                long v = Math.round(value);
                if (v < SMALLINT_MIN || v > SMALLINT_MAX) {
                    throw new YppException("Line " + line + ": smallint value " + v
                        + " out of range [" + SMALLINT_MIN + ", " + SMALLINT_MAX + "]");
                }
                yield (double) v;
            }
            case "integer" -> {
                long v = Math.round(value);
                if (v < INTEGER_MIN || v > INTEGER_MAX) {
                    throw new YppException("Line " + line + ": integer value " + v
                        + " out of range [" + INTEGER_MIN + ", " + INTEGER_MAX + "]");
                }
                yield (double) v;
            }
            case "double" -> {
                if (value < DOUBLE_MIN || value > DOUBLE_MAX) {
                    throw new YppException("Line " + line + ": double value " + value
                        + " out of range [" + DOUBLE_MIN + ", " + DOUBLE_MAX + "]");
                }
                // Truncate to 15 decimal digits
                yield truncateToScale(value, DOUBLE_SCALE);
            }
            default -> value;
        };
    }

    // ------------------------------------------------------------------
    // Formatting helpers
    // ------------------------------------------------------------------

    private static String formatDouble(double value) {
        // Round to 15 significant figures, then strip trailing zeros
        java.math.BigDecimal bd = new java.math.BigDecimal(value)
            .round(new java.math.MathContext(15))
            .stripTrailingZeros();
        String s = bd.toPlainString();
        // Ensure there is a decimal point for doubles
        if (!s.contains(".")) s += ".0";
        return s;
    }

    // ------------------------------------------------------------------
    // Networking & Method Chaining Runtime Engine
    // ------------------------------------------------------------------

    private void execWhile(ASTNode.WhileNode n) {
        while (isTruthy(evalExprRaw(n.condition))) {
            for (ASTNode stmt : n.body) {
                execStatement(stmt);
            }
        }
    }

    private boolean isTruthy(Object val) {
        if (val == null) return false;
        if (val instanceof Boolean b) return b;
        if (val instanceof Number num) return num.doubleValue() != 0;
        if (val instanceof String s) return !s.isEmpty() && !s.equalsIgnoreCase("false") && !s.equalsIgnoreCase("null");
        return true;
    }

    private Object evalNot(ASTNode.NotExprNode n) {
        Object val = evalExprRaw(n.expr);
        return !isTruthy(val);
    }

    private Object evalInlineAssign(ASTNode.InlineAssignExprNode n) {
        Object val = evalExprRaw(n.expr);
        globals.put(n.varName, val);
        return val;
    }

    private Object evalNewObject(ASTNode.NewObjectExprNode n) {
        List<Object> args = new ArrayList<>();
        for (ASTNode.ExprNode argNode : n.args) {
            args.add(evalExprRaw(argNode));
        }

        String className = n.className;
        if ("Network".equalsIgnoreCase(className)) {
            if (!importedPackages.contains("ynetworking")) {
                throw new YppException("Line " + n.line + ": Import ynetworking is required to use Network.");
            }
            String host = String.valueOf(args.get(0));
            int port = ((Number) args.get(1)).intValue();
            try {
                java.net.Socket socket = new java.net.Socket(host, port);
                return new YppNetworkObject(socket);
            } catch (Exception ex) {
                throw new YppException("Line " + n.line + ": Network connection to " + host + ":" + port + " failed: " + ex.getMessage());
            }
        } else if ("Server".equalsIgnoreCase(className)) {
            if (!importedPackages.contains("ynetworking")) {
                throw new YppException("Line " + n.line + ": Import ynetworking is required to use Server.");
            }
            int port = ((Number) args.get(0)).intValue();
            try {
                java.net.ServerSocket ss = new java.net.ServerSocket(port);
                return new YppServerObject(ss);
            } catch (Exception ex) {
                throw new YppException("Line " + n.line + ": Server failed on port " + port + ": " + ex.getMessage());
            }
        } else if ("primitivedataStream".equalsIgnoreCase(className)) {
            if (!args.isEmpty() && args.get(0) instanceof YppStreamObject streamObj) {
                return streamObj;
            }
            if (!args.isEmpty() && args.get(0) instanceof YppNetworkObject netObj) {
                return new YppStreamObject(netObj.getSocket());
            }
            return new YppStreamObject(args.isEmpty() ? null : args.get(0));
        } else if ("reader".equalsIgnoreCase(className) || "userinput".equalsIgnoreCase(className)) {
            return new YppConsoleReaderObject();
        }

        throw new YppException("Line " + n.line + ": Unknown object or class '" + className + "'");
    }

    private Object evalMethodCall(ASTNode.MethodCallExprNode n) {
        Object targetObj = evalExprRaw(n.target);
        List<Object> args = new ArrayList<>();
        for (ASTNode.ExprNode argNode : n.args) {
            args.add(evalExprRaw(argNode));
        }

        if (targetObj instanceof YppObject obj) {
            return obj.callMethod(n.methodName, args, this, n.line);
        } else if (targetObj instanceof String str) {
            if ("equals".equalsIgnoreCase(n.methodName) && !args.isEmpty()) {
                return String.valueOf(str).equals(String.valueOf(args.get(0)));
            } else if ("readline".equalsIgnoreCase(n.methodName) || "readutf-8".equalsIgnoreCase(n.methodName)) {
                return str;
            } else if ("length".equalsIgnoreCase(n.methodName)) {
                return (double) str.length();
            }
        }

        throw new YppException("Line " + n.line + ": Cannot invoke method '" + n.methodName + "' on " + (targetObj == null ? "null" : targetObj.getClass().getSimpleName()));
    }

    // ------------------------------------------------------------------
    // Y++ Object Interfaces & Wrappers
    // ------------------------------------------------------------------

    public interface YppObject {
        Object callMethod(String methodName, List<Object> args, Interpreter interp, int line);
    }

    public static class YppNetworkObject implements YppObject {
        private final java.net.Socket socket;
        public YppNetworkObject(java.net.Socket socket) { this.socket = socket; }
        public java.net.Socket getSocket() { return socket; }

        @Override
        public Object callMethod(String methodName, List<Object> args, Interpreter interp, int line) {
            try {
                if ("outstream".equalsIgnoreCase(methodName)) {
                    return new YppStreamObject(socket.getOutputStream());
                } else if ("inputstream".equalsIgnoreCase(methodName)) {
                    return new YppStreamObject(socket.getInputStream());
                } else if ("close".equalsIgnoreCase(methodName)) {
                    socket.close();
                    return null;
                }
            } catch (Exception ex) {
                throw new YppException("Line " + line + ": Network error in " + methodName + ": " + ex.getMessage());
            }
            throw new YppException("Line " + line + ": Unknown method '" + methodName + "' on Network");
        }
    }

    public static class YppServerObject implements YppObject {
        private final java.net.ServerSocket serverSocket;
        public YppServerObject(java.net.ServerSocket serverSocket) { this.serverSocket = serverSocket; }

        @Override
        public Object callMethod(String methodName, List<Object> args, Interpreter interp, int line) {
            try {
                if ("accept".equalsIgnoreCase(methodName)) {
                    java.net.Socket client = serverSocket.accept();
                    return new YppNetworkObject(client);
                } else if ("close".equalsIgnoreCase(methodName)) {
                    serverSocket.close();
                    return null;
                }
            } catch (Exception ex) {
                throw new YppException("Line " + line + ": Server error in " + methodName + ": " + ex.getMessage());
            }
            throw new YppException("Line " + line + ": Unknown method '" + methodName + "' on Server");
        }
    }

    public static class YppStreamObject implements YppObject {
        private java.io.InputStream in;
        private java.io.OutputStream out;
        private java.io.BufferedReader reader;
        private java.io.PrintWriter writer;

        public YppStreamObject(Object streamOrSocket) {
            try {
                if (streamOrSocket instanceof java.net.Socket s) {
                    this.in = s.getInputStream();
                    this.out = s.getOutputStream();
                } else if (streamOrSocket instanceof java.io.InputStream is) {
                    this.in = is;
                } else if (streamOrSocket instanceof java.io.OutputStream os) {
                    this.out = os;
                } else if (streamOrSocket instanceof YppStreamObject so) {
                    this.in = so.in;
                    this.out = so.out;
                }
                if (in != null && reader == null) reader = new java.io.BufferedReader(new java.io.InputStreamReader(in, java.nio.charset.StandardCharsets.UTF_8));
                if (out != null && writer == null) writer = new java.io.PrintWriter(new java.io.OutputStreamWriter(out, java.nio.charset.StandardCharsets.UTF_8), true);
            } catch (Exception ignored) {}
        }

        @Override
        public Object callMethod(String methodName, List<Object> args, Interpreter interp, int line) {
            try {
                if ("utf-8".equalsIgnoreCase(methodName) || "writeutf-8".equalsIgnoreCase(methodName) || "write".equalsIgnoreCase(methodName)) {
                    String msg = args.isEmpty() ? "" : String.valueOf(args.get(0));
                    if (writer != null) {
                        writer.println(msg);
                    }
                    return null;
                } else if ("readutf-8".equalsIgnoreCase(methodName) || "readline".equalsIgnoreCase(methodName) || "read".equalsIgnoreCase(methodName)) {
                    if (reader != null) {
                        String lineVal = reader.readLine();
                        return lineVal != null ? lineVal : "";
                    }
                    return "";
                } else if ("close".equalsIgnoreCase(methodName)) {
                    if (in != null) in.close();
                    if (out != null) out.close();
                    return null;
                }
            } catch (Exception ex) {
                throw new YppException("Line " + line + ": Stream error in " + methodName + ": " + ex.getMessage());
            }
            throw new YppException("Line " + line + ": Unknown method '" + methodName + "' on primitivedataStream");
        }
    }

    public static class YppConsoleReaderObject implements YppObject {
        @Override
        public Object callMethod(String methodName, List<Object> args, Interpreter interp, int line) {
            if ("readline".equalsIgnoreCase(methodName) || "read".equalsIgnoreCase(methodName)) {
                String input = interp.inputProvider.readLine("");
                return input != null ? input : "";
            }
            throw new YppException("Line " + line + ": Unknown method '" + methodName + "' on reader");
        }
    }
}
