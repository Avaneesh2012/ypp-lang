// Y++ AST node definitions
package ypp;

import java.util.List;

/** Base type for all AST nodes. */
public abstract class ASTNode {

    // ------------------------------------------------------------------
    // Statements
    // ------------------------------------------------------------------

    /** The whole program — a list of top-level statements. */
    public static class Program extends ASTNode {
        public final List<ASTNode> statements;
        public Program(List<ASTNode> statements) { this.statements = statements; }
    }

    /** Import apple *   or   Import apple {ClassName} */
    public static class ImportNode extends ASTNode {
        public final String  pkg;
        public final boolean importAll;   // true  → *
        public final String  className;   // null  → *
        public ImportNode(String pkg, boolean importAll, String className) {
            this.pkg = pkg; this.importAll = importAll; this.className = className;
        }
    }

    /**
     * NUM block:
     *   NUM 1 { ... }
     *   Hello { ... }
     */
    public static class NumBlockNode extends ASTNode {
        public final String          label;  // "1", "2", "Hello", …
        public final List<ASTNode>   statements;
        public final int             line;
        public NumBlockNode(String label, List<ASTNode> statements, int line) {
            this.label = label; this.statements = statements; this.line = line;
        }
    }

    /**
     * Variable declaration inside a NUM block:
     *   integer applecount = 3i
     *   smallint = 1si          ← name == type keyword (shorthand)
     *   double appleweight = 2.4d
     */
    public static class VarDeclNode extends ASTNode {
        public final String      typeName;  // "smallint" | "integer" | "double"
        public final String      varName;   // may equal typeName for shorthand
        public final ExprNode    value;
        public final int         line;
        public VarDeclNode(String typeName, String varName, ExprNode value, int line) {
            this.typeName = typeName; this.varName = varName;
            this.value = value; this.line = line;
        }
    }

    /**
     * STRING block:
     *   STRING 1 { ... }
     */
    public static class StringBlockNode extends ASTNode {
        public final String                label;  // "1", "2", "Hello", ...
        public final List<ASTNode>         statements;
        public final int                   line;
        public StringBlockNode(String label, List<ASTNode> statements, int line) {
            this.label = label; this.statements = statements; this.line = line;
        }
    }

    /**
     * Variable declaration inside a STRING block:
     *   slong fun = "fun"
     *   schar dumb = "d"
     */
    public static class StringVarDeclNode extends ASTNode {
        public final String   typeName;  // "slong" | "schar"
        public final String   varName;
        public final ExprNode value;
        public final int      line;
        public StringVarDeclNode(String typeName, String varName, ExprNode value, int line) {
            this.typeName = typeName; this.varName = varName;
            this.value = value; this.line = line;
        }
    }

    /**
     * PRINT: <expr> ;
     * PRINT: int()  <expr> ;
     * PRINT: double() <expr> ;
     * PRINT: string() <expr> ;
     * PRINT: stringint() <expr> ;
     */
    public static class PrintNode extends ASTNode {
        public enum Cast { NONE, INT, DOUBLE, STRING, STRINGINT }
        public final Cast     cast;
        public final ExprNode expr;
        public PrintNode(Cast cast, ExprNode expr) { this.cast = cast; this.expr = expr; }
    }

    /**
     * EXCEPTION CONCAT :: stmt1 :: stmt2 ...
     */
    public static class ExceptionConcatNode extends ASTNode {
        public final List<ASTNode> statements;
        public ExceptionConcatNode(List<ASTNode> statements) { this.statements = statements; }
    }

    /**
     * Global assignment (outside NUM / STRING block):
     *   together = (NUM 1)applecount * (NUM 1)appleweight;
     *   together = (STRING 1)fun + (STRING 1)dumb;
     */
    public static class AssignNode extends ASTNode {
        public final String   name;
        public final ExprNode value;
        public AssignNode(String name, ExprNode value) { this.name = name; this.value = value; }
    }

    // ------------------------------------------------------------------
    // Function System
    // ------------------------------------------------------------------

    public static class Param {
        public final String type;
        public final String name;
        public Param(String type, String name) { this.type = type; this.name = name; }
    }

    /** func name(params) { body } */
    public static class FuncDeclNode extends ASTNode {
        public final String name;
        public final List<Param> params;
        public final List<ASTNode> body;
        public final int line;
        public FuncDeclNode(String name, List<Param> params, List<ASTNode> body, int line) {
            this.name = name; this.params = params; this.body = body; this.line = line;
        }
    }

    /** NEW funcName.(blockType blockLabel) = aliasName; OR NEW funcName = aliasName; */
    public static class NewAliasNode extends ASTNode {
        public final String funcName;
        public final String blockType;  // "NUM" or "STRING", null if whole func
        public final String blockLabel; // e.g. "1", null if whole func
        public final String aliasName;
        public final int line;
        public NewAliasNode(String funcName, String blockType, String blockLabel, String aliasName, int line) {
            this.funcName = funcName; this.blockType = blockType; this.blockLabel = blockLabel; this.aliasName = aliasName; this.line = line;
        }
    }

    /** global { innerBlock } */
    public static class GlobalBlockNode extends ASTNode {
        public final ASTNode innerBlock; // NumBlockNode or StringBlockNode or PrintNode
        public GlobalBlockNode(ASTNode innerBlock) { this.innerBlock = innerBlock; }
    }

    /** aliasName(); */
    public static class FuncCallNode extends ASTNode {
        public final String aliasName;
        public final int line;
        public FuncCallNode(String aliasName, int line) { this.aliasName = aliasName; this.line = line; }
    }

    /** slong.name = name; (binding a parameter) */
    public static class ParamBindNode extends ASTNode {
        public final String typeName;
        public final String paramName;
        public final String paramRef;
        public final int line;
        public ParamBindNode(String typeName, String paramName, String paramRef, int line) {
            this.typeName = typeName; this.paramName = paramName; this.paramRef = paramRef; this.line = line;
        }
    }

    /** name.input("prompt"); */
    public static class ParamInputNode extends ASTNode {
        public final String paramName;
        public final ExprNode prompt;
        public final int line;
        public ParamInputNode(String paramName, ExprNode prompt, int line) {
            this.paramName = paramName; this.prompt = prompt; this.line = line;
        }
    }

    /** name.next(); */
    public static class ParamNextNode extends ASTNode {
        public final String paramName;
        public ParamNextNode(String paramName) { this.paramName = paramName; }
    }

    /** name.break; */
    public static class ParamBreakNode extends ASTNode {
        public final String paramName;
        public ParamBreakNode(String paramName) { this.paramName = paramName; }
    }

    /** while (condition) { body } */
    public static class WhileNode extends ASTNode {
        public final ExprNode condition;
        public final List<ASTNode> body;
        public final int line;
        public WhileNode(ExprNode condition, List<ASTNode> body, int line) {
            this.condition = condition; this.body = body; this.line = line;
        }
    }

    // ------------------------------------------------------------------
    // Expressions
    // ------------------------------------------------------------------

    public abstract static class ExprNode extends ASTNode {}

    /** A number literal — always stored as double internally. */
    public static class NumberLiteralNode extends ExprNode {
        public final double  value;
        public final String  typeName;  // "smallint" | "integer" | "double"
        public NumberLiteralNode(double value, String typeName) {
            this.value = value; this.typeName = typeName;
        }
    }

    /** A string literal. */
    public static class StringLiteralNode extends ExprNode {
        public final String value;
        public StringLiteralNode(String value) { this.value = value; }
    }

    /** A plain identifier (global variable). */
    public static class IdentNode extends ExprNode {
        public final String name;
        public IdentNode(String name) { this.name = name; }
    }

    /**
     * Accesses a variable from a named NUM block:
     *   (NUM 1)applecount
     *   (Hello)varname
     */
    public static class NumAccessNode extends ExprNode {
        public final String blockLabel;  // "1" or "Hello"
        public final String varName;
        public NumAccessNode(String blockLabel, String varName) {
            this.blockLabel = blockLabel; this.varName = varName;
        }
    }

    /**
     * Accesses a variable from a named STRING block:
     *   (STRING 1)fun
     *   (Hello)varname
     */
    public static class StringAccessNode extends ExprNode {
        public final String blockLabel;  // "1" or "Hello"
        public final String varName;
        public StringAccessNode(String blockLabel, String varName) {
            this.blockLabel = blockLabel; this.varName = varName;
        }
    }

    /** Binary arithmetic: left op right  where op ∈ { + - * / } */
    public static class BinaryExprNode extends ExprNode {
        public final ExprNode left;
        public final char     op;
        public final ExprNode right;
        public BinaryExprNode(ExprNode left, char op, ExprNode right) {
            this.left = left; this.op = op; this.right = right;
        }
    }

    /** target.methodName(args...) */
    public static class MethodCallExprNode extends ExprNode {
        public final ExprNode target;
        public final String methodName;
        public final List<ExprNode> args;
        public final int line;
        public MethodCallExprNode(ExprNode target, String methodName, List<ExprNode> args, int line) {
            this.target = target; this.methodName = methodName; this.args = args; this.line = line;
        }
    }

    /** new ClassName(args...) */
    public static class NewObjectExprNode extends ExprNode {
        public final String blockType;
        public final String blockLabel;
        public final String className;
        public final List<ExprNode> args;
        public final int line;
        public NewObjectExprNode(String blockType, String blockLabel, String className, List<ExprNode> args, int line) {
            this.blockType = blockType; this.blockLabel = blockLabel; this.className = className; this.args = args; this.line = line;
        }
    }

    /** NOT: expr or !expr */
    public static class NotExprNode extends ExprNode {
        public final ExprNode expr;
        public final int line;
        public NotExprNode(ExprNode expr, int line) { this.expr = expr; this.line = line; }
    }

    /** line = expr */
    public static class InlineAssignExprNode extends ExprNode {
        public final String varName;
        public final ExprNode expr;
        public final int line;
        public InlineAssignExprNode(String varName, ExprNode expr, int line) {
            this.varName = varName; this.expr = expr; this.line = line;
        }
    }
}
