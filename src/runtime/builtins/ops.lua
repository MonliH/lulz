function _lulz_add(pos, l, r)
    if type(l) == "string" and type(r) == "string" then
        return l..r
    else
        return l + r
    end
end

function _lulz_sub(pos, l, r)
    return l - r
end

function _lulz_mul(pos, l, r)
    return l * r
end

function _lulz_div(pos, l, r)
    return l / r
end

function _lulz_mod(pos, l, r)
    return l % r
end

function _lulz_and(pos, l, r)
    return l and r
end

function _lulz_or(pos, l, r)
    return l or r
end

function _lulz_eq(pos, l, r)
    return l == r
end

function _lulz_neq(pos, l, r)
    return l ~= r
end

function _lulz_gt(pos, l, r)
    return l > r
end

function _lulz_lt(pos, l, r)
    return l < r
end

function _lulz_gte(pos, l, r)
    return l >= r
end

function _lulz_lte(pos, l, r)
    return l <= r
end

