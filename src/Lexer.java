// Y++ Lexer — converts raw source into a list of Tokens
package ypp;

import java.util.ArrayList;
import java.util.List;

public class Lexer {

    private final String source;
    private int pos  = 0;
    private int line = 1;

    public Lexer(String source) {
        // Strip block comments  \\...\\  before tokenising
        this.source = stripComments(source);
    }

    // ---------------------------------------------------------------
    // Public API
    // ---------------------------------------------------------------

    public List<Token> tokenize() {
        List<Token> tokens = new ArrayList<>();
        while (pos < source.length()) {
            skipWhitespace();
            if (pos >= source.length()) break;

            Token t = nextToken();
            if (t != null) tokens.add(t);
        }
        tokens.add(new Token(Token.Type.EOF, "", line));
        return tokens;
    }

    // ---------------------------------------------------------------
    // Comment stripping  \\...\\
    // ---------------------------------------------------------------

    private static String stripComments(String src) {
        StringBuilder sb = new StringBuilder();
        int i = 0;
        while (i < src.length()) {
            if (i + 1 < src.length() && src.charAt(i) == '\\' && src.charAt(i + 1) == '\\') {
                // Find closing \\
                int end = src.indexOf("\\\\", i + 2);
                if (end == -1) {
                    // Unclosed comment — skip to end
                    break;
                }
                // Replace comment text with spaces to preserve line numbers
                for (int j = i; j < end + 2; j++) {
                    sb.append(src.charAt(j) == '\n' ? '\n' : ' ');
                }
                i = end + 2;
            } else {
                sb.append(src.charAt(i));
                i++;
            }
        }
        return sb.toString();
    }

    // ---------------------------------------------------------------
    // Core tokeniser
    // ---------------------------------------------------------------

    private void skipWhitespace() {
        while (pos < source.length()) {
            char c = source.charAt(pos);
            if (c == '\n') { line++; pos++; }
            else if (Character.isWhitespace(c)) pos++;
            else break;
        }
    }

    private Token nextToken() {
        int startLine = line;
        char c = source.charAt(pos);

        // String literal
        if (c == '"') return readString(startLine);

        // Number literal (may carry si / i / d suffix)
        if (Character.isDigit(c) || (c == '-' && pos + 1 < source.length() && Character.isDigit(source.charAt(pos + 1)))) {
            return readNumber(startLine);
        }

        // Identifier / keyword
        if (Character.isLetter(c) || c == '_') return readIdent(startLine);

        // Single/double character symbols
        pos++;
        return switch (c) {
            case '=' -> new Token(Token.Type.EQUALS,    "=", startLine);
            case '*' -> new Token(Token.Type.STAR,      "*", startLine);
            case '+' -> new Token(Token.Type.PLUS,      "+", startLine);
            case '-' -> new Token(Token.Type.MINUS,     "-", startLine);
            case '/' -> new Token(Token.Type.SLASH,     "/", startLine);
            case '.' -> new Token(Token.Type.DOT,       ".", startLine);
            case '!' -> new Token(Token.Type.BANG,      "!", startLine);
            case ':' -> {
                if (pos < source.length() && source.charAt(pos) == ':') {
                    pos++;
                    yield new Token(Token.Type.DOUBLE_COLON, "::", startLine);
                }
                yield new Token(Token.Type.COLON, ":", startLine);
            }
            case ';' -> new Token(Token.Type.SEMICOLON, ";", startLine);
            case ',' -> new Token(Token.Type.COMMA,     ",", startLine);
            case '{' -> new Token(Token.Type.LBRACE,    "{", startLine);
            case '}' -> new Token(Token.Type.RBRACE,    "}", startLine);
            case '(' -> new Token(Token.Type.LPAREN,    "(", startLine);
            case ')' -> new Token(Token.Type.RPAREN,    ")", startLine);
            default  -> null;   // skip unrecognised characters silently
        };
    }

    // ---------------------------------------------------------------
    // String  "..."
    // ---------------------------------------------------------------

    private Token readString(int startLine) {
        pos++; // skip opening "
        StringBuilder sb = new StringBuilder();
        while (pos < source.length() && source.charAt(pos) != '"') {
            if (source.charAt(pos) == '\n') line++;
            sb.append(source.charAt(pos++));
        }
        if (pos < source.length()) pos++; // skip closing "
        return new Token(Token.Type.LIT_STRING, sb.toString(), startLine);
    }

    // ---------------------------------------------------------------
    // Numbers:  2.4d   3i   1si   (bare numbers used as labels)
    // ---------------------------------------------------------------

    private Token readNumber(int startLine) {
        StringBuilder sb = new StringBuilder();
        // Optional leading minus
        if (source.charAt(pos) == '-') sb.append(source.charAt(pos++));

        while (pos < source.length() && (Character.isDigit(source.charAt(pos)) || source.charAt(pos) == '.')) {
            sb.append(source.charAt(pos++));
        }

        // Read optional suffix: si / i / d
        String suffix = readSuffix();
        String numText = sb.toString();

        return switch (suffix) {
            case "si" -> new Token(Token.Type.LIT_SMALLINT, numText, startLine);
            case "i"  -> new Token(Token.Type.LIT_INTEGER,  numText, startLine);
            case "d"  -> new Token(Token.Type.LIT_DOUBLE,   numText, startLine);
            default   -> new Token(Token.Type.NUMBER_LABEL, numText, startLine);
        };
    }

    /** Peek ahead and consume  si / i / d  suffix if present. */
    private String readSuffix() {
        if (pos >= source.length()) return "";
        // "si" must come before "i" check
        if (pos + 1 < source.length()
                && source.charAt(pos) == 's'
                && source.charAt(pos + 1) == 'i') {
            pos += 2;
            return "si";
        }
        char c = source.charAt(pos);
        if (c == 'i' || c == 'd') {
            pos++;
            return String.valueOf(c);
        }
        return "";
    }

    // ---------------------------------------------------------------
    // Identifiers & Keywords
    // ---------------------------------------------------------------

    private Token readIdent(int startLine) {
        StringBuilder sb = new StringBuilder();
        while (pos < source.length() && (Character.isLetterOrDigit(source.charAt(pos)) || source.charAt(pos) == '_')) {
            sb.append(source.charAt(pos++));
        }
        String word = sb.toString();

        // Check for cast calls: int(), double(), string(), stringint()
        // They appear as bare words immediately followed by ()
        if ((word.equals("int") || word.equals("double") || word.equals("string") || word.equals("stringint")) && peekTwoChars("()")) {
            pos += 2; // consume ()
            Token.Type ctype = switch (word) {
                case "int" -> Token.Type.CAST_INT;
                case "double" -> Token.Type.CAST_DOUBLE;
                case "string" -> Token.Type.CAST_STRING;
                default -> Token.Type.CAST_STRINGINT;
            };
            return new Token(ctype, word + "()", startLine);
        }

        return switch (word) {
            case "Import"    -> new Token(Token.Type.IMPORT,       word, startLine);
            case "PRINT"     -> new Token(Token.Type.PRINT,        word, startLine);
            case "NUM"       -> new Token(Token.Type.NUM,          word, startLine);
            case "STRING"    -> new Token(Token.Type.STRING_BLOCK, word, startLine);
            case "EXCEPTION" -> new Token(Token.Type.EXCEPTION,    word, startLine);
            case "CONCAT"    -> new Token(Token.Type.CONCAT,       word, startLine);
            case "Public"    -> new Token(Token.Type.PUBLIC,       word, startLine);
            case "class"     -> new Token(Token.Type.CLASS,        word, startLine);
            case "func"      -> new Token(Token.Type.FUNC,         word, startLine);
            case "NEW", "new"-> new Token(Token.Type.NEW,          word, startLine);
            case "global"    -> new Token(Token.Type.KW_GLOBAL,    word, startLine);
            case "while"     -> new Token(Token.Type.WHILE,        word, startLine);
            case "NOT"       -> new Token(Token.Type.NOT,          word, startLine);
            case "smallint"  -> new Token(Token.Type.KW_SMALLINT,  word, startLine);
            case "integer"   -> new Token(Token.Type.KW_INTEGER,   word, startLine);
            case "double"    -> new Token(Token.Type.KW_DOUBLE,    word, startLine);
            case "slong"     -> new Token(Token.Type.KW_SLONG,     word, startLine);
            case "schar"     -> new Token(Token.Type.KW_SCHAR,     word, startLine);
            default          -> new Token(Token.Type.IDENT,        word, startLine);
        };
    }

    private boolean peekTwoChars(String s) {
        return pos + 1 < source.length()
            && source.charAt(pos) == s.charAt(0)
            && source.charAt(pos + 1) == s.charAt(1);
    }
}
