// Y++ Token definitions
package ypp;

public class Token {

    public enum Type {
        // Keywords
        IMPORT,         // Import
        IMPORT_ALL,     // *  (after Import pkg)
        PRINT,          // PRINT
        NUM,            // NUM
        STRING_BLOCK,   // STRING (block keyword)
        EXCEPTION,      // EXCEPTION
        CONCAT,         // CONCAT
        PUBLIC,         // Public
        CLASS,          // class
        FUNC,           // func
        NEW,            // NEW
        KW_GLOBAL,      // global
        WHILE,          // while
        NOT,            // NOT / NOT:
        BANG,           // !

        // Type keywords
        KW_SMALLINT,    // smallint
        KW_INTEGER,     // integer
        KW_DOUBLE,      // double
        KW_SLONG,       // slong
        KW_SCHAR,       // schar

        // Cast calls (e.g.  int()  double()  string()  stringint() )
        CAST_INT,       // int()
        CAST_DOUBLE,    // double()
        CAST_STRING,    // string()
        CAST_STRINGINT, // stringint()

        // Literals
        LIT_SMALLINT,   // e.g.  1si
        LIT_INTEGER,    // e.g.  3i
        LIT_DOUBLE,     // e.g.  2.4d
        LIT_STRING,     // e.g.  "hello"

        // Identifiers / labels
        IDENT,          // any bare word
        NUMBER_LABEL,   // bare integer used as a NUM label  e.g. 1, 2

        // Operators
        EQUALS,         // =
        STAR,           // *
        PLUS,           // +
        MINUS,          // -
        SLASH,          // /

        // Punctuation
        COLON,          // :
        DOUBLE_COLON,   // ::
        SEMICOLON,      // ;
        COMMA,          // ,
        DOT,            // .
        LBRACE,         // {
        RBRACE,         // }
        LPAREN,         // (
        RPAREN,         // )

        EOF
    }

    public final Type   type;
    public final String value;   // raw text
    public final int    line;

    public Token(Type type, String value, int line) {
        this.type  = type;
        this.value = value;
        this.line  = line;
    }

    @Override
    public String toString() {
        return "Token(" + type + ", " + value + ", line=" + line + ")";
    }
}
