// Y++ Recursive-descent Parser
package ypp;

import java.util.ArrayList;
import java.util.List;

import static ypp.Token.Type.*;

public class Parser {

    private final List<Token> tokens;
    private int pos = 0;

    public Parser(List<Token> tokens) {
        this.tokens = tokens;
    }

    // ---------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------

    private Token peek() {
        return tokens.get(pos);
    }

    private Token peek(int offset) {
        int idx = pos + offset;
        return idx < tokens.size() ? tokens.get(idx) : tokens.get(tokens.size() - 1);
    }

    private Token consume() {
        Token t = tokens.get(pos);
        if (t.type != EOF) pos++;
        return t;
    }

    private Token expect(Token.Type type) {
        Token t = consume();
        if (t.type != type) {
            throw new YppException("Line " + t.line + ": expected " + type + " but got " + t.type + " ('" + t.value + "')");
        }
        return t;
    }

    private boolean check(Token.Type type) {
        return peek().type == type;
    }

    private boolean match(Token.Type type) {
        if (check(type)) { consume(); return true; }
        return false;
    }

    private void optionalSemicolon() {
        if (check(SEMICOLON)) consume();
    }

    // ---------------------------------------------------------------
    // Entry point
    // ---------------------------------------------------------------

    public ASTNode.Program parse() {
        List<ASTNode> stmts = new ArrayList<>();
        while (!check(EOF)) {
            ASTNode s = parseStatement();
            if (s != null) stmts.add(s);
        }
        return new ASTNode.Program(stmts);
    }

    // ---------------------------------------------------------------
    // Statement dispatcher
    // ---------------------------------------------------------------

    private ASTNode parseStatement() {
        Token t = peek();
        return switch (t.type) {
            case IMPORT       -> parseImport();
            case PRINT        -> parsePrint();
            case NUM          -> parseNumBlock();
            case STRING_BLOCK -> parseStringBlock();
            case EXCEPTION    -> parseExceptionConcat();
            case FUNC         -> parseFuncDecl();
            case NEW          -> parseNewAlias();
            case KW_GLOBAL    -> { consume(); yield new ASTNode.GlobalBlockNode(parseStatement()); }
            case WHILE        -> parseWhile();
            case KW_SMALLINT,
                 KW_INTEGER,
                 KW_DOUBLE    -> parseBareVarDecl();
            case KW_SLONG,
                 KW_SCHAR     -> {
                     if (peek(1).type == DOT) yield parseParamBind();
                     yield parseBareStringVarDecl();
                 }
            case IDENT        -> {
                if (peek(1).type == LBRACE) yield parseNamedBlock();
                if (peek(1).type == DOT && (peek(2).value.equals("input") || peek(2).value.equals("next") || peek(2).value.equals("break"))) yield parseParamAction();
                if (peek(1).type == LPAREN && peek(2).type == RPAREN && (peek(3).type == SEMICOLON || peek(3).type == RBRACE || peek(3).type == EOF)) yield parseFuncCall();
                if (peek(1).type == EQUALS && peek(2).type == NEW) yield parseAliasAssignment();
                if (peek(1).type == EQUALS) yield parseAssignment();
                yield parseExprStatement();
            }
            default -> { consume(); yield null; }           // skip unknown tokens
        };
    }

    private ASTNode parseExceptionConcat() {
        expect(EXCEPTION);
        expect(CONCAT);
        // Accept both   EXCEPTION CONCAT() { ... }   and   EXCEPTION CONCAT { ... }
        if (check(LPAREN)) {
            consume(); // (
            expect(RPAREN); // )
        }
        expect(LBRACE);

        List<ASTNode> stmts = new ArrayList<>();
        while (!check(RBRACE) && !check(EOF)) {
            ASTNode stmt = parseStatement();
            if (stmt != null) stmts.add(stmt);
        }

        expect(RBRACE);
        return new ASTNode.ExceptionConcatNode(stmts);
    }

    // ---------------------------------------------------------------
    // Function System
    // ---------------------------------------------------------------

    private ASTNode parseFuncDecl() {
        int line = peek().line;
        expect(FUNC);
        String name = expect(IDENT).value;
        expect(LPAREN);
        List<ASTNode.Param> params = new ArrayList<>();
        while (!check(RPAREN) && !check(EOF)) {
            String pType = consume().value;
            String pName = expect(IDENT).value;
            params.add(new ASTNode.Param(pType, pName));
            if (check(COMMA)) consume();
        }
        expect(RPAREN);
        expect(LBRACE);
        List<ASTNode> body = new ArrayList<>();
        while (!check(RBRACE) && !check(EOF)) {
            ASTNode stmt = parseStatement();
            if (stmt != null) body.add(stmt);
        }
        expect(RBRACE);
        return new ASTNode.FuncDeclNode(name, params, body, line);
    }

    private ASTNode parseNewAlias() {
        int line = peek().line;
        expect(NEW);
        String funcName = expect(IDENT).value;
        
        String blockType = null;
        String blockLabel = null;
        
        if (check(DOT)) {
            consume();
            expect(LPAREN);
            blockType = consume().value; // NUM or STRING
            StringBuilder labelSb = new StringBuilder();
            while (!check(RPAREN) && !check(EOF)) {
                if (labelSb.length() > 0) labelSb.append(" ");
                labelSb.append(consume().value);
            }
            blockLabel = labelSb.toString().trim();
            expect(RPAREN);
        }
        
        expect(EQUALS);
        String aliasName = expect(IDENT).value;
        optionalSemicolon();
        
        return new ASTNode.NewAliasNode(funcName, blockType, blockLabel, aliasName, line);
    }

    private ASTNode parseParamBind() {
        int line = peek().line;
        String type = consume().value; // slong / schar
        expect(DOT);
        String paramName = expect(IDENT).value;
        expect(EQUALS);
        String paramRef = expect(IDENT).value;
        optionalSemicolon();
        return new ASTNode.ParamBindNode(type, paramName, paramRef, line);
    }

    private ASTNode parseParamAction() {
        int line = peek().line;
        String paramName = expect(IDENT).value;
        expect(DOT);
        String action = expect(IDENT).value;
        
        if (action.equals("input")) {
            expect(LPAREN);
            ASTNode.ExprNode prompt = parseExpr();
            expect(RPAREN);
            if (!check(SEMICOLON)) {
                throw new YppException("Line " + line + ": missing ';' after " + paramName + ".input(...) — semicolons are required on input calls.");
            }
            consume();
            return new ASTNode.ParamInputNode(paramName, prompt, line);
        } else if (action.equals("next")) {
            expect(LPAREN);
            expect(RPAREN);
            if (!check(SEMICOLON)) {
                throw new YppException("Line " + line + ": missing ';' after " + paramName + ".next() — semicolons are required.");
            }
            consume();
            return new ASTNode.ParamNextNode(paramName);
        } else if (action.equals("break")) {
            if (!check(SEMICOLON)) {
                throw new YppException("Line " + line + ": missing ';' after " + paramName + ".break — semicolons are required.");
            }
            consume();
            return new ASTNode.ParamBreakNode(paramName);
        } else {
            throw new YppException("Line " + line + ": unknown action '" + action + "' on parameter '" + paramName + "'.");
        }
    }

    private ASTNode parseFuncCall() {
        int line = peek().line;
        String aliasName = expect(IDENT).value;
        expect(LPAREN);
        expect(RPAREN);
        optionalSemicolon();
        return new ASTNode.FuncCallNode(aliasName, line);
    }

    private ASTNode parseNamedBlock() {
        int blockLine = peek().line;
        String label = consume().value; // label name
        expect(LBRACE);

        List<ASTNode> stmts = new ArrayList<>();
        while (!check(RBRACE) && !check(EOF)) {
            ASTNode stmt = parseStatement();
            if (stmt != null) stmts.add(stmt);
        }
        expect(RBRACE);

        // Fallback to NumBlockNode if neither NUM nor STRING is provided
        return new ASTNode.NumBlockNode(label, stmts, blockLine);
    }

    // ---------------------------------------------------------------
    // Import
    // ---------------------------------------------------------------

    private ASTNode.ImportNode parseImport() {
        expect(IMPORT);
        String pkg = expect(IDENT).value;

        if (check(STAR)) {
            consume();
            return new ASTNode.ImportNode(pkg, true, null);
        }
        if (check(LBRACE)) {
            consume();
            String cls = expect(IDENT).value;
            expect(RBRACE);
            return new ASTNode.ImportNode(pkg, false, cls);
        }
        // bare import — treat as import-all
        return new ASTNode.ImportNode(pkg, true, null);
    }

    // ---------------------------------------------------------------
    // PRINT
    // ---------------------------------------------------------------

    private ASTNode.PrintNode parsePrint() {
        expect(PRINT);
        expect(COLON);

        // Optional cast
        ASTNode.PrintNode.Cast cast = ASTNode.PrintNode.Cast.NONE;
        if (check(CAST_INT)) {
            consume(); cast = ASTNode.PrintNode.Cast.INT;
        } else if (check(CAST_DOUBLE)) {
            consume(); cast = ASTNode.PrintNode.Cast.DOUBLE;
        } else if (check(CAST_STRING)) {
            consume(); cast = ASTNode.PrintNode.Cast.STRING;
        } else if (check(CAST_STRINGINT)) {
            consume(); cast = ASTNode.PrintNode.Cast.STRINGINT;
        }

        ASTNode.ExprNode expr = parseExpr();
        optionalSemicolon();
        return new ASTNode.PrintNode(cast, expr);
    }

    // ---------------------------------------------------------------
    // NUM block:   NUM <anything> { decls/statements }
    // ---------------------------------------------------------------

    private ASTNode.NumBlockNode parseNumBlock() {
        int blockLine = peek().line;
        expect(NUM);

        String label = readUntilBrace();

        expect(LBRACE);
        List<ASTNode> stmts = new ArrayList<>();
        while (!check(RBRACE) && !check(EOF)) {
            ASTNode stmt = parseStatement();
            if (stmt != null) stmts.add(stmt);
        }
        expect(RBRACE);

        return new ASTNode.NumBlockNode(label, stmts, blockLine);
    }

    private ASTNode.VarDeclNode parseVarDecl() {
        Token typeTok = peek();
        if (typeTok.type != KW_SMALLINT && typeTok.type != KW_INTEGER && typeTok.type != KW_DOUBLE) {
            consume();
            return null;
        }
        int declLine = typeTok.line;
        String typeName = consume().value;

        String varName;
        if (check(IDENT)) {
            varName = consume().value;
        } else {
            varName = typeName;
        }

        expect(EQUALS);
        ASTNode.ExprNode value = parseLiteralExpr();
        match(COMMA);
        match(SEMICOLON);

        return new ASTNode.VarDeclNode(typeName, varName, value, declLine);
    }

    private ASTNode parseBareVarDecl() {
        int declLine = peek().line;
        String typeName = consume().value;

        String varName;
        if (check(IDENT)) { varName = consume().value; } else { varName = typeName; }

        expect(EQUALS);
        ASTNode.ExprNode value = parseLiteralExpr();
        match(COMMA);
        match(SEMICOLON);

        return new ASTNode.VarDeclNode(typeName, varName, value, declLine);
    }

    // ---------------------------------------------------------------
    // STRING block:   STRING <anything> { decls/statements }
    // ---------------------------------------------------------------

    private ASTNode.StringBlockNode parseStringBlock() {
        int blockLine = peek().line;
        expect(STRING_BLOCK);

        // Handle bare declaration e.g.  STRING line;  or  STRING line = "val";
        if (check(IDENT) && (peek(1).type == SEMICOLON || peek(1).type == EQUALS)) {
            String varName = consume().value;
            ASTNode.ExprNode val = new ASTNode.StringLiteralNode("");
            if (match(EQUALS)) val = parseExpr();
            optionalSemicolon();
            return new ASTNode.StringBlockNode(null, List.of(new ASTNode.StringVarDeclNode("slong", varName, val, blockLine)), blockLine);
        }

        String label = readUntilBrace();

        expect(LBRACE);
        List<ASTNode> stmts = new ArrayList<>();
        while (!check(RBRACE) && !check(EOF)) {
            ASTNode stmt = parseStatement();
            if (stmt != null) stmts.add(stmt);
        }
        expect(RBRACE);

        return new ASTNode.StringBlockNode(label, stmts, blockLine);
    }

    private String readUntilBrace() {
        StringBuilder sb = new StringBuilder();
        while (!check(LBRACE) && !check(EOF)) {
            if (sb.length() > 0) sb.append(" ");
            sb.append(consume().value);
        }
        return sb.toString().trim();
    }

    /**
     * Variable declaration inside a STRING block:
     *   slong fun = "fun";
     *   schar dumb = "d";
     */
    private ASTNode.StringVarDeclNode parseStringVarDecl() {
        Token typeTok = peek();
        if (typeTok.type != KW_SLONG && typeTok.type != KW_SCHAR) {
            consume();
            return null;
        }
        int declLine = typeTok.line;
        String typeName = consume().value; // "slong" or "schar"

        String varName;
        if (check(IDENT)) {
            varName = consume().value;
        } else {
            varName = typeName;
        }

        expect(EQUALS);
        ASTNode.ExprNode value = parseLiteralExpr();

        match(COMMA);
        match(SEMICOLON);

        return new ASTNode.StringVarDeclNode(typeName, varName, value, declLine);
    }

    /**
     * Parse a bare string var declaration outside any block — syntax error ("no string pair error").
     */
    private ASTNode parseBareStringVarDecl() {
        int declLine = peek().line;
        String typeName = consume().value;

        String varName;
        if (check(IDENT)) { varName = consume().value; } else { varName = typeName; }

        expect(EQUALS);
        ASTNode.ExprNode value = parseLiteralExpr();
        match(COMMA);
        match(SEMICOLON);

        return new ASTNode.StringVarDeclNode(typeName, varName, value, declLine);
    }

    // ---------------------------------------------------------------
    // Global assignment:   together = expr;
    // ---------------------------------------------------------------

    private ASTNode parseAliasAssignment() {
        int line = peek().line;
        String aliasName = expect(IDENT).value;
        expect(EQUALS);
        expect(NEW);
        
        String blockType = null;
        String blockLabel = null;
        String funcName = null;
        
        if (check(LPAREN)) {
            // alias = new (NUM 1).examples(...)
            consume(); // (
            if (check(NUM)) { blockType = "NUM"; consume(); }
            else if (check(STRING_BLOCK)) { blockType = "STRING"; consume(); }
            
            StringBuilder labelSb = new StringBuilder();
            while (!check(RPAREN) && !check(EOF)) {
                if (labelSb.length() > 0) labelSb.append(" ");
                labelSb.append(consume().value);
            }
            blockLabel = labelSb.toString().trim();
            expect(RPAREN);
            expect(DOT);
            funcName = expect(IDENT).value;
        } else {
            // alias = new examples(...)
            funcName = expect(IDENT).value;
            if (check(DOT)) {
                // alias = new examples.(NUM 1)
                consume();
                expect(LPAREN);
                if (check(NUM)) { blockType = "NUM"; consume(); }
                else if (check(STRING_BLOCK)) { blockType = "STRING"; consume(); }
                StringBuilder labelSb = new StringBuilder();
                while (!check(RPAREN) && !check(EOF)) {
                    if (labelSb.length() > 0) labelSb.append(" ");
                    labelSb.append(consume().value);
                }
                blockLabel = labelSb.toString().trim();
                expect(RPAREN);
            }
        }
        
        // Optional parameter list (...)
        if (check(LPAREN)) {
            consume();
            while (!check(RPAREN) && !check(EOF)) {
                consume();
            }
            expect(RPAREN);
        }
        
        optionalSemicolon();
        return new ASTNode.NewAliasNode(funcName, blockType, blockLabel, aliasName, line);
    }

    private ASTNode.AssignNode parseAssignment() {
        String name = expect(IDENT).value;
        expect(EQUALS);
        ASTNode.ExprNode expr = parseExpr();
        optionalSemicolon();
        return new ASTNode.AssignNode(name, expr);
    }

    private ASTNode parseWhile() {
        int line = peek().line;
        expect(WHILE);
        expect(LPAREN);
        ASTNode.ExprNode cond = parseExpr();
        expect(RPAREN);
        expect(LBRACE);
        List<ASTNode> body = new ArrayList<>();
        while (!check(RBRACE) && !check(EOF)) {
            ASTNode stmt = parseStatement();
            if (stmt != null) body.add(stmt);
        }
        expect(RBRACE);
        return new ASTNode.WhileNode(cond, body, line);
    }

    private ASTNode parseExprStatement() {
        ASTNode.ExprNode expr = parseExpr();
        optionalSemicolon();
        return expr;
    }

    // ---------------------------------------------------------------
    // Expressions  (additive → multiplicative → unary/primary)
    // ---------------------------------------------------------------

    private ASTNode.ExprNode parseExpr() {
        int line = peek().line;

        // NOT: expr or !expr
        if (check(NOT) || check(BANG)) {
            consume();
            if (check(COLON)) consume();
            return new ASTNode.NotExprNode(parseExpr(), line);
        }

        // Inline assignment:  var = expr
        if (check(IDENT) && peek(1).type == EQUALS) {
            String varName = consume().value;
            consume(); // =
            return new ASTNode.InlineAssignExprNode(varName, parseExpr(), line);
        }

        return parseAdditive();
    }

    private ASTNode.ExprNode parseAdditive() {
        ASTNode.ExprNode left = parseMultiplicative();
        while (check(PLUS) || check(MINUS)) {
            char op = consume().value.charAt(0);
            ASTNode.ExprNode right = parseMultiplicative();
            left = new ASTNode.BinaryExprNode(left, op, right);
        }
        return left;
    }

    private ASTNode.ExprNode parseMultiplicative() {
        ASTNode.ExprNode left = parsePrimary();
        while (check(STAR) || check(SLASH)) {
            char op = consume().value.charAt(0);
            ASTNode.ExprNode right = parsePrimary();
            left = new ASTNode.BinaryExprNode(left, op, right);
        }
        return left;
    }

    /**
     * Primary:
     *   (NUM 1)applecount   → NumAccessNode
     *   (STRING 1)fun       → StringAccessNode
     *   (Hello)varname      → NumAccessNode
     *   "string"            → StringLiteralNode
     *   1si / 3i / 2.4d    → NumberLiteralNode
     *   <ident>             → IdentNode
     */
    private ASTNode.ExprNode parsePrimary() {
        ASTNode.ExprNode expr = parsePrimaryBase();

        // Method call chaining:  expr.methodName(args...)
        while (check(DOT)) {
            int line = peek().line;
            consume(); // .
            String methodName = expect(IDENT).value;
            List<ASTNode.ExprNode> args = new ArrayList<>();
            if (check(LPAREN)) {
                consume(); // (
                while (!check(RPAREN) && !check(EOF)) {
                    args.add(parseExpr());
                    if (check(COMMA)) consume();
                }
                expect(RPAREN);
            }
            expr = new ASTNode.MethodCallExprNode(expr, methodName, args, line);
        }
        return expr;
    }

    private ASTNode.ExprNode parsePrimaryBase() {
        Token t = peek();

        if (t.type == NEW) {
            return parseNewObjectExpr();
        }

        // Parenthesized expression or block access:  (NUM 1)var  or  (expr)
        if (t.type == LPAREN) {
            if (peek(1).type == NUM || peek(1).type == STRING_BLOCK || (peek(1).type == NUMBER_LABEL && peek(2).type == RPAREN && peek(3).type == IDENT)) {
                return parseBlockAccess();
            } else {
                consume(); // (
                ASTNode.ExprNode expr = parseExpr();
                expect(RPAREN);
                return expr;
            }
        }

        // String literal
        if (t.type == LIT_STRING) {
            consume();
            return new ASTNode.StringLiteralNode(t.value);
        }

        // Numeric literals
        if (t.type == LIT_SMALLINT) {
            consume();
            return new ASTNode.NumberLiteralNode(Double.parseDouble(t.value), "smallint");
        }
        if (t.type == LIT_INTEGER) {
            consume();
            return new ASTNode.NumberLiteralNode(Double.parseDouble(t.value), "integer");
        }
        if (t.type == LIT_DOUBLE) {
            consume();
            return new ASTNode.NumberLiteralNode(Double.parseDouble(t.value), "double");
        }
        if (t.type == NUMBER_LABEL) {
            consume();
            return new ASTNode.NumberLiteralNode(Double.parseDouble(t.value), "double");
        }

        // Bare identifier or function/constructor call e.g. userinput() or reader(userinput())
        if (t.type == IDENT) {
            int line = t.line;
            String name = consume().value;
            if (check(LPAREN)) {
                consume(); // (
                List<ASTNode.ExprNode> args = new ArrayList<>();
                while (!check(RPAREN) && !check(EOF)) {
                    args.add(parseExpr());
                    if (check(COMMA)) consume();
                }
                expect(RPAREN);
                return new ASTNode.NewObjectExprNode(null, null, name, args, line);
            }
            return new ASTNode.IdentNode(name);
        }

        throw new YppException("Line " + t.line + ": unexpected token '" + t.value + "' in expression");
    }

    private ASTNode.ExprNode parseNewObjectExpr() {
        int line = peek().line;
        expect(NEW);
        String blockType = null;
        String blockLabel = null;
        String className = null;

        if (check(LPAREN)) {
            consume(); // (
            if (check(NUM)) { blockType = "NUM"; consume(); }
            else if (check(STRING_BLOCK)) { blockType = "STRING"; consume(); }
            StringBuilder labelSb = new StringBuilder();
            while (!check(RPAREN) && !check(EOF)) {
                if (labelSb.length() > 0) labelSb.append(" ");
                labelSb.append(consume().value);
            }
            blockLabel = labelSb.toString().trim();
            expect(RPAREN);
            expect(DOT);
            className = expect(IDENT).value;
        } else {
            className = expect(IDENT).value;
        }

        List<ASTNode.ExprNode> args = new ArrayList<>();
        if (check(LPAREN)) {
            consume();
            while (!check(RPAREN) && !check(EOF)) {
                args.add(parseExpr());
                if (check(COMMA)) consume();
            }
            expect(RPAREN);
        }

        return new ASTNode.NewObjectExprNode(blockType, blockLabel, className, args, line);
    }

    /**
     * Literal-only expression used inside NUM blocks.
     * Grammar:  LIT_SMALLINT | LIT_INTEGER | LIT_DOUBLE | LIT_STRING
     */
    private ASTNode.ExprNode parseLiteralExpr() {
        Token t = peek();
        if (t.type == LIT_SMALLINT) { consume(); return new ASTNode.NumberLiteralNode(Double.parseDouble(t.value), "smallint"); }
        if (t.type == LIT_INTEGER)  { consume(); return new ASTNode.NumberLiteralNode(Double.parseDouble(t.value), "integer");  }
        if (t.type == LIT_DOUBLE)   { consume(); return new ASTNode.NumberLiteralNode(Double.parseDouble(t.value), "double");   }
        if (t.type == LIT_STRING)   { consume(); return new ASTNode.StringLiteralNode(t.value); }
        throw new YppException("Line " + t.line + ": expected literal value, got '" + t.value + "'");
    }

    /**
     * Parse  (NUM <anything>)varname  or  (STRING <anything>)varname  or  (<anything>)varname
     */
    private ASTNode.ExprNode parseBlockAccess() {
        expect(LPAREN);

        boolean isStringBlock = false;
        if (check(NUM)) {
            consume(); // skip NUM keyword
        } else if (check(STRING_BLOCK)) {
            consume(); // skip STRING keyword
            isStringBlock = true;
        }

        StringBuilder labelSb = new StringBuilder();
        while (!check(RPAREN) && !check(EOF)) {
            if (labelSb.length() > 0) labelSb.append(" ");
            labelSb.append(consume().value);
        }
        String label = labelSb.toString().trim();

        expect(RPAREN);
        String varName = expect(IDENT).value;

        if (isStringBlock) {
            return new ASTNode.StringAccessNode(label, varName);
        } else {
            return new ASTNode.NumAccessNode(label, varName);
        }
    }
}
