# Y++ Parser - Builds Abstract Syntax Tree from tokens

include("lexer.jl")

# AST Node Types
abstract type ASTNode end

struct NumberNode <: ASTNode
    value::Float64
end

struct StringNode <: ASTNode
    value::String
end

struct IdentifierNode <: ASTNode
    name::String
end

struct VariableDeclaration <: ASTNode
    var_type::String  # "nvar" or "svar"
    name::String
    value::ASTNode
end

struct PrintStatement <: ASTNode
    expression::ASTNode
end

struct FunctionDeclaration <: ASTNode
    name::String
    parameters::Vector{Tuple{String, String}}  # (type, name) pairs
    body::Vector{ASTNode}
end

struct FunctionCall <: ASTNode
    object::Union{String, Nothing}  # For func.run(hi) syntax
    function_name::String
    arguments::Vector{ASTNode}
end

struct AssignmentStatement <: ASTNode
    name::String
    value::ASTNode
end

struct Program <: ASTNode
    statements::Vector{ASTNode}
end

mutable struct Parser
    tokens::Vector{Token}
    position::Int
    current_token::Token
    
    function Parser(tokens::Vector{Token})
        parser = new(tokens, 1, tokens[1])
        return parser
    end
end

function advance!(parser::Parser)
    parser.position += 1
    if parser.position <= length(parser.tokens)
        parser.current_token = parser.tokens[parser.position]
    end
end

function expect!(parser::Parser, token_type::TokenType)
    if parser.current_token.type != token_type
        error("Expected $(token_type), got $(parser.current_token.type) at line $(parser.current_token.line)")
    end
    token = parser.current_token
    advance!(parser)
    return token
end

function parse_primary(parser::Parser)::ASTNode
    token = parser.current_token
    
    if token.type == NUMBER
        advance!(parser)
        return NumberNode(token.value)
    elseif token.type == STRING
        advance!(parser)
        return StringNode(token.value)
    elseif token.type == IDENTIFIER
        name = token.value
        advance!(parser)
        
        # Check for function call with dot notation (func.run(hi))
        if parser.current_token.type == DOT
            advance!(parser)  # Skip dot
            method_name = expect!(parser, IDENTIFIER).value
            expect!(parser, LPAREN)
            
            args = ASTNode[]
            if parser.current_token.type != RPAREN
                push!(args, parse_primary(parser))
                while parser.current_token.type == COMMA
                    advance!(parser)
                    push!(args, parse_primary(parser))
                end
            end
            
            expect!(parser, RPAREN)
            return FunctionCall(name, method_name, args)
        end
        
        return IdentifierNode(name)
    elseif token.type == LPAREN
        advance!(parser)
        expr = parse_primary(parser)
        expect!(parser, RPAREN)
        return expr
    else
        error("Unexpected token: $(token.type) at line $(token.line)")
    end
end

function parse_variable_declaration(parser::Parser)::VariableDeclaration
    var_type = parser.current_token.value  # "nvar" or "svar"
    advance!(parser)
    
    name = expect!(parser, IDENTIFIER).value
    expect!(parser, EQUALS)
    
    value = parse_primary(parser)
    expect!(parser, SEMICOLON)
    
    return VariableDeclaration(var_type, name, value)
end

function parse_print_statement(parser::Parser)::PrintStatement
    advance!(parser)  # Skip PRINT
    expect!(parser, DOT) # Skip colon (we treat it as dot in lexer)
    
    expr = parse_primary(parser)
    expect!(parser, SEMICOLON)
    
    return PrintStatement(expr)
end

function parse_function_declaration(parser::Parser)::FunctionDeclaration
    advance!(parser)  # Skip Func
    
    func_name = expect!(parser, IDENTIFIER).value
    expect!(parser, LBRACKET)
    
    # Parse parameters
    parameters = Tuple{String, String}[]
    if parser.current_token.type != RBRACKET
        # Parse first parameter
        param_type = parser.current_token.value
        advance!(parser)
        param_name = expect!(parser, IDENTIFIER).value
        push!(parameters, (param_type, param_name))
        
        # Parse remaining parameters
        while parser.current_token.type == COMMA
            advance!(parser)
            param_type = parser.current_token.value
            advance!(parser)
            param_name = expect!(parser, IDENTIFIER).value
            push!(parameters, (param_type, param_name))
        end
    end
    
    expect!(parser, RBRACKET)
    expect!(parser, LBRACE)
    
    # Parse function body
    body = ASTNode[]
    while parser.current_token.type != RBRACE && parser.current_token.type != EOF
        stmt = parse_statement(parser)
        if stmt !== nothing
            push!(body, stmt)
        end
    end
    
    expect!(parser, RBRACE)
    
    return FunctionDeclaration(func_name, parameters, body)
end

function parse_assignment(parser::Parser)::AssignmentStatement
    name = parser.current_token.value
    advance!(parser)
    expect!(parser, EQUALS)
    
    value = parse_primary(parser)
    expect!(parser, SEMICOLON)
    
    return AssignmentStatement(name, value)
end

function parse_statement(parser::Parser)::Union{ASTNode, Nothing}
    token = parser.current_token
    
    if token.type == NVAR || token.type == SVAR
        return parse_variable_declaration(parser)
    elseif token.type == PRINT
        return parse_print_statement(parser)
    elseif token.type == FUNC
        return parse_function_declaration(parser)
    elseif token.type == IDENTIFIER
        # Check if it's an assignment or function call
        next_pos = parser.position + 1
        if next_pos <= length(parser.tokens)
            next_token = parser.tokens[next_pos]
            if next_token.type == EQUALS
                return parse_assignment(parser)
            elseif next_token.type == DOT
                # Function call
                stmt = parse_primary(parser)
                expect!(parser, SEMICOLON)
                return stmt
            end
        end
        # Just an identifier expression
        advance!(parser)
        return IdentifierNode(token.value)
    elseif token.type == EOF
        return nothing
    else
        error("Unexpected token in statement: $(token.type) at line $(token.line)")
    end
end

function parse(tokens::Vector{Token})::Program
    parser = Parser(tokens)
    statements = ASTNode[]
    
    while parser.current_token.type != EOF
        stmt = parse_statement(parser)
        if stmt !== nothing
            push!(statements, stmt)
        end
    end
    
    return Program(statements)
end

# Made with Bob
