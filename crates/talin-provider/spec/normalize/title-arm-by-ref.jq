# Titles an arm of a `oneOf` union with the name of the component it references.
#
# `{ "allOf": [{ "$ref": "#/components/schemas/Foo" }] }` ->
# `{ "allOf": [{ "$ref": "#/components/schemas/Foo" }], "title": "Foo" }`.
#
# Runs over every `oneOf` union, wherever it is written.

walk(
  if type == "object" and (.oneOf | type) == "array"
  then .oneOf |= map(
    if type != "object" then .
    else
      ([.["$ref"]?, (.allOf? // [])[]["$ref"]?] | map(select(type == "string")) | first) as $ref
      | if $ref == null then . else .title = ($ref | split("/") | last) end
    end
  )
  else .
  end
)
