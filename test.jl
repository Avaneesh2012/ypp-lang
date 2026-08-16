# Y++ Test Suite

include("interpreter.jl")

function test_lexer()
    println("Testing Lexer...")
    
    # Test basic tokenization
    code = "nvar x = 10;"
    tokens = tokenize(code)
    @assert tokens[1].type == NVAR
    @assert tokens[2].type == IDENTIFIER
    @assert tokens[3].type == EQUALS
    @assert tokens[4].type == NUMBER
    @assert tokens[5].type == SEMICOLON
    
    println("✓ Lexer tests passed")
end

function test_parser()
    println("Testing Parser...")
    
    # Test variable declaration
    code = "nvar x = 10;"
    tokens = tokenize(code)
    ast = parse(tokens)
    @assert length(ast.statements) == 1
    @assert ast.statements[1] isa VariableDeclaration
    
    println("✓ Parser tests passed")
end

function test_interpreter()
    println("Testing Interpreter...")
    
    # Test number variable
    code1 = "nvar x = 42;"
    env1 = Environment()
    ast1 = parse(tokenize(code1))
    evaluate(ast1, env1)
    @assert get_variable(env1, "x") == 42.0
    
    # Test string variable
    code2 = "svar name = \"Julia\";"
    env2 = Environment()
    ast2 = parse(tokenize(code2))
    evaluate(ast2, env2)
    @assert get_variable(env2, "name") == "Julia"
    
    println("✓ Interpreter tests passed")
end

function test_print()
    println("Testing Print Statement...")
    
    code = """
    svar msg = "Hello, Y++!";
    PRINT: msg;
    """
    
    println("Expected output: Hello, Y++!")
    println("Actual output: ", end="")
    interpret(code)
    
    println("✓ Print test completed")
end

function test_function()
    println("Testing Functions...")
    
    code = """
    nvar result = 0;
    Func test[nvar a, svar b] {
        a = 100;
        b = "modified";
    }
    test.run(result);
    """
    
    # This should execute without errors
    interpret(code)
    
    println("✓ Function test completed")
end

function test_example_file()
    println("Testing example.ypp...")
    
    if isfile("example.ypp")
        println("Expected output: cool kid!")
        println("Actual output: ", end="")
        interpret_file("example.ypp")
        println("✓ Example file test completed")
    else
        println("⚠ example.ypp not found, skipping")
    end
end

function run_all_tests()
    println("=" ^ 50)
    println("Y++ Compiler Test Suite")
    println("=" ^ 50)
    println()
    
    try
        test_lexer()
        println()
        
        test_parser()
        println()
        
        test_interpreter()
        println()
        
        test_print()
        println()
        
        test_function()
        println()
        
        test_example_file()
        println()
        
        println("=" ^ 50)
        println("All tests completed successfully! ✓")
        println("=" ^ 50)
    catch e
        println()
        println("=" ^ 50)
        println("Test failed with error:")
        println(e)
        println("=" ^ 50)
        rethrow(e)
    end
end

# Run tests if this is the main script
if abspath(PROGRAM_FILE) == @__FILE__
    run_all_tests()
end

# Made with Bob
