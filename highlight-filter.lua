-- Robust highlight removal filter for pandoc
-- Removes highlight spans from the AST

local function has_class(classes, name)
    if classes then
        for _, c in ipairs(classes) do
            if c == name then
                return true
            end
        end
    end
    return false
end

-- Remove highlight spans
function Span(el)
    if has_class(el.classes, 'hl') or has_class(el.classes, 'mark') then
        return el.content
    end
    return el
end

-- Process inline elements
function Inlines(inlines)
    local result = {}
    for i, inline in ipairs(inlines) do
        if inline.t == "Span" then
            if has_class(inline.classes, 'highlight') then
                -- Flatten: add all content from highlight span
                for _, child in ipairs(inline.content) do
                    table.insert(result, child)
                end
            else
                table.insert(result, inline)
            end
        else
            table.insert(result, inline)
        end
    end
    return result
end
