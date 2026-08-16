# Y++ Compiler Main Entry Point

include("interpreter.jl")

function main()
    if length(ARGS) < 1
        println("Usage: julia ypp.jl <filename.ypp>")
        println("   or: julia ypp.jl -e \"<code>\"")
        exit(1)
    end
    
    if ARGS[1] == "-e"
        # Execute code directly
        if length(ARGS) < 2
            println("Error: No code provided after -e flag")
            exit(1)
        end
        code = ARGS[2]
        try
            interpret(code)
        catch e
            println("Error: ", e)
            exit(1)
        end
    else
        # Execute file
        filename = ARGS[1]
        if !isfile(filename)
            println("Error: File '$filename' not found")
            exit(1)
        end
        
        try
            interpret_file(filename)
        catch e
            println("Error: ", e)
            exit(1)
        end
    end
end

# Run main if this is the main script
if abspath(PROGRAM_FILE) == @__FILE__
    main()
end

# Made with Bob
