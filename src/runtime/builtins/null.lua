local _lulz_null_metatable = {}
_lulz_null_metatable.__index = _lulz_null_metatable

function _lulz_null_metatable.ToString(self)
    return "NOOB"
end

function _lulz_NewNull()
    local obj = {}
    return setmetatable(obj, _lulz_null_metatable)
end

local original_type = type
type = function(obj)
    local otype = original_type(obj)
    if otype == "table" and getmetatable(obj) == _lulz_null_metatable then
        return "lulz_null"
    end
    return otype
end

function _lulz_check_variable(pos, name, var)
    if type(var) == "nil" then
        _ffi_lulz_error(pos, "variable `" .. name .. "` is undefined")
    else
        return var
    end
end
