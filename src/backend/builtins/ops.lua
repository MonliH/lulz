function _lulz_add(l, r)
    if type(l) == "string" and type(r) == "string" then
        return l..r
    else
        return l + r
    end
end

function _lulz_sub(l, r)
    return l - r
end

function _lulz_mul(l, r)
    return l * r
end

function _lulz_div(l, r)
    return l / r
end

function _lulz_mod(l, r)
    return l % r
end

function _lulz_and(l, r)
    return l and r
end

function _lulz_or(l, r)
    return l or r
end

function _lulz_eq(l, r)
    return l == r
end

function _lulz_neq(l, r)
    return l ~= r
end

function _lulz_gt(l, r)
    return l > r
end

function _lulz_lt(l, r)
    return l < r
end

function _lulz_gte(l, r)
    return l >= r
end

function _lulz_lte(l, r)
    return l <= r
end

