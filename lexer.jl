# Y++ Lexer - Tokenizes Y++ source code

@enum TokenType begin
    # Keywords
    NVAR
    SVAR
    PRINT
    FUNC
    
    # Literals
    NUMBER
    STRING
    IDENTIFIER
    
    # Operators and Punctuation
    EQUALS
    SEMICOLON
    COMMA
    LPAREN
    RPAREN
    LBRACE
    RBRACE
    LBRACKET
    RBRACKET
    DOT
    
    # Special
    EOF
    NEWLINE
end

struct Token
    type::TokenType
    value::Any
    line::Int
    column::Int
end

mutable struct Lexer
    input::String
    position::Int
    line::Int
    column::Int
    current_char::Union{Char, Nothing}
    
    function Lexer(input::String)
        lexer = new(input, 1, 1, 1, nothing)
        if length(input) > 0
            lexer.current_char = input[1]
        end
        return lexer
    end
end

function advance!(lexer::Lexer)
    if lexer.position >= length(lexer.input)
        lexer.current_char = nothing
        return
    end
    
    if lexer.current_char == '\n'
        lexer.line += 1
        lexer.column = 1
    else
        lexer.column += 1
    end
    
    lexer.position += 1
    if lexer.position <= length(lexer.input)
        lexer.current_char = lexer.input[lexer.position]
    else
        lexer.current_char = nothing
    end
end

function peek(lexer::Lexer, offset::Int=1)::Union{Char, Nothing}
    peek_pos = lexer.position + offset
    if peek_pos <= length(lexer.input)
        return lexer.input[peek_pos]
    end
    return nothing
end

function skip_whitespace!(lexer::Lexer)
    while lexer.current_char !== nothing && lexer.current_char in [' ', '\t', '\r']
        advance!(lexer)
    end
end

function read_number(lexer::Lexer)::Float64
    num_str = ""
    while lexer.current_char !== nothing && (isdigit(lexer.current_char) || lexer.current_char == '.')
        num_str *= lexer.current_char
        advance!(lexer)
    end
    return parse(Float64, num_str)
end

function read_string(lexer::Lexer)::String
    str = ""
    advance!(lexer)  # Skip opening quote
    
    while lexer.current_char !== nothing && lexer.current_char != '"'
        str *= lexer.current_char
        advance!(lexer)
    end
    
    if lexer.current_char == '"'
        advance!(lexer)  # Skip closing quote
    end
    
    return str
end

function read_identifier(lexer::Lexer)::String
    id = ""
    while lexer.current_char !== nothing && (isalnum(lexer.current_char) || lexer.current_char == '_')
        id *= lexer.current_char
        advance!(lexer)
    end
    return id
end

function get_next_token(lexer::Lexer)::Token
    while lexer.current_char !== nothing
        # Skip whitespace
        if lexer.current_char in [' ', '\t', '\r']
            skip_whitespace!(lexer)
            continue
        end
        
        # Handle newlines
        if lexer.current_char == '\n'
            line, col = lexer.line, lexer.column
            advance!(lexer)
            return Token(NEWLINE, "\n", line, col)
        end
        
        # Handle numbers
        if isdigit(lexer.current_char)
            line, col = lexer.line, lexer.column
            return Token(NUMBER, read_number(lexer), line, col)
        end
        
        # Handle strings
        if lexer.current_char == '"'
            line, col = lexer.line, lexer.column
            return Token(STRING, read_string(lexer), line, col)
        end
        
        # Handle identifiers and keywords
        if isalpha(lexer.current_char) || lexer.current_char == '_'
            line, col = lexer.line, lexer.column
            id = read_identifier(lexer)
            
            # Check for keywords
            token_type = if id == "nvar"
                NVAR
            elseif id == "svar"
                SVAR
            elseif id == "PRINT"
                PRINT
            elseif id == "Func"
                FUNC
            else
                IDENTIFIER
            end
            
            return Token(token_type, id, line, col)
        end
        
        # Single character tokens
        line, col = lexer.line, lexer.column
        char = lexer.current_char
        
        if char == '='
            advance!(lexer)
            return Token(EQUALS, "=", line, col)
        elseif char == ';'
            advance!(lexer)
            return Token(SEMICOLON, ";", line, col)
        elseif char == ','
            advance!(lexer)
            return Token(COMMA, ",", line, col)
        elseif char == '('
            advance!(lexer)
            return Token(LPAREN, "(", line, col)
        elseif char == ')'
            advance!(lexer)
            return Token(RPAREN, ")", line, col)
        elseif char == '{'
            advance!(lexer)
            return Token(LBRACE, "{", line, col)
        elseif char == '}'
            advance!(lexer)
            return Token(RBRACE, "}", line, col)
        elseif char == '['
            advance!(lexer)
            return Token(LBRACKET, "[", line, col)
        elseif char == ']'
            advance!(lexer)
            return Token(RBRACKET, "]", line, col)
        elseif char == '.'
            advance!(lexer)
            return Token(DOT, ".", line, col)
        elseif char == ':'
            advance!(lexer)
            continue  # Skip colons
        else
            error("Unexpected character: '$char' at line $(lexer.line), column $(lexer.column)")
        end
    end
    
    return Token(EOF, nothing, lexer.line, lexer.column)
end

function tokenize(input::String)::Vector{Token}
    lexer = Lexer(input)
    tokens = Token[]
    
    while true
        token = get_next_token(lexer)
        if token.type != NEWLINE  # Skip newline tokens for simplicity
            push!(tokens, token)
        end
        if token.type == EOF
            break
        end
    end
    
    return tokens
end

# Made with Bob
