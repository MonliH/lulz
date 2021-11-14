function table.pack(...)
  return { n = select("#", ...), ... }
end

function __lulz_to_str(v)
    local ty = type(v)
    if ty == "number" or ty == "string" then
        return v
    elseif ty == "boolean" then
        return v and "WIN" or "FAIL"
    elseif ty == "lulz_null" then
        return "NOOB"
    end
end

function _lulz_println(...)
    local args = table.pack(...)
    for i=1,args.n do
        io.write(__lulz_to_str(args[i]))
    end
    print()
end

function _lulz_print(...)
    local args = table.pack(...)
    for i=1,args.n do
        io.write(__lulz_to_str(args[i]))
    end
end
