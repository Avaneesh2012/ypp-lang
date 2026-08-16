# Y++ Interpreter - Evaluates the AST

include("parser.jl")

mutable struct Environment
    variables::Dict{String, Any}
    functions::Dict{String, FunctionDeclaration}
    parent::Union{Environment, Nothing}
    
    function Environment(parent::Union{Environment, Nothing}=nothing)
        new(Dict{String, Any}(), Dict{String, FunctionDeclaration}(), parent)
    end
end

function set_variable!(env::Environment, name::String, value::Any)
    env.variables[name] = value
end

function get_variable(env::Environment, name::String)::Any
    if haskey(env.variables, name)
        return env.variables[name]
    elseif env.parent !== nothing
        return get_variable(env.parent, name)
    else
        error("Undefined variable: $name")
    end
end

function set_function!(env::Environment, name::String, func::FunctionDeclaration)
    env.functions[name] = func
end

function get_function(env::Environment, name::String)::Union{FunctionDeclaration, Nothing}
    if haskey(env.functions, name)
        return env.functions[name]
    elseif env.parent !== nothing
        return get_function(env.parent, name)
    else
        return nothing
    end
end

function evaluate(node::NumberNode, env::Environment)::Float64
    return node.value
end

function evaluate(node::StringNode, env::Environment)::String
    return node.value
end

function evaluate(node::IdentifierNode, env::Environment)::Any
    return get_variable(env, node.name)
end

function evaluate(node::VariableDeclaration, env::Environment)::Nothing
    value = evaluate(node.value, env)
    
    # Type checking
    if node.var_type == "nvar" && !(value isa Number)
        error("Type error: nvar $(node.name) must be assigned a number")
    elseif node.var_type == "svar" && !(value isa String)
        error("Type error: svar $(node.name) must be assigned a string")
    end
    
    set_variable!(env, node.name, value)
    return nothing
end

function evaluate(node::AssignmentStatement, env::Environment)::Nothing
    value = evaluate(node.value, env)
    set_variable!(env, node.name, value)
    return nothing
end

function evaluate(node::PrintStatement, env::Environment)::Nothing
    value = evaluate(node.expression, env)
    println(value)
    return nothing
end

function evaluate(node::FunctionDeclaration, env::Environment)::Nothing
    set_function!(env, node.name, node)
    return nothing
end

function evaluate(node::FunctionCall, env::Environment)::Any
    # Handle func.run(args) syntax
    if node.object !== nothing
        # Get the function object
        func_obj = get_variable(env, node.object)
        
        if func_obj isa FunctionDeclaration
            # Create new environment for function execution
            func_env = Environment(env)
            
            # Bind parameters
            if length(node.arguments) != length(func_obj.parameters)
                error("Function $(func_obj.name) expects $(length(func_obj.parameters)) arguments, got $(length(node.arguments))")
            end
            
            for (i, (param_type, param_name)) in enumerate(func_obj.parameters)
                arg_value = evaluate(node.arguments[i], env)
                
                # Type checking for parameters
                if param_type == "nvar" && !(arg_value isa Number)
                    error("Type error: parameter $param_name must be a number")
                elseif param_type == "svar" && !(arg_value isa String)
                    error("Type error: parameter $param_name must be a string")
                end
                
                set_variable!(func_env, param_name, arg_value)
            end
            
            # Execute function body
            result = nothing
            for stmt in func_obj.body
                result = evaluate(stmt, func_env)
            end
            
            return result
        else
            error("$(node.object) is not a function")
        end
    else
        # Direct function call (not implemented in the example, but included for completeness)
        func = get_function(env, node.function_name)
        if func === nothing
            error("Undefined function: $(node.function_name)")
        end
        
        # Create new environment for function execution
        func_env = Environment(env)
        
        # Bind parameters
        if length(node.arguments) != length(func.parameters)
            error("Function $(func.name) expects $(length(func.parameters)) arguments, got $(length(node.arguments))")
        end
        
        for (i, (param_type, param_name)) in enumerate(func.parameters)
            arg_value = evaluate(node.arguments[i], env)
            set_variable!(func_env, param_name, arg_value)
        end
        
        # Execute function body
        result = nothing
        for stmt in func.body
            result = evaluate(stmt, func_env)
        end
        
        return result
    end
end

function evaluate(node::Program, env::Environment)::Nothing
    for statement in node.statements
        evaluate(statement, env)
    end
    return nothing
end

function interpret(source_code::String)
    # Tokenize
    tokens = tokenize(source_code)
    
    # Parse
    ast = parse(tokens)
    
    # Evaluate
    global_env = Environment()
    evaluate(ast, global_env)
end

function interpret_file(filename::String)
    source_code = read(filename, String)
    interpret(source_code)
end

# Made with Bob
