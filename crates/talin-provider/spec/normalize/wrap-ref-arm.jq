# Wraps a `$ref` arm of a `oneOf` union inside an `allOf`.
#
# `{ "$ref": "#/components/schemas/SomeSchema" }` ->
# `{ "allOf": [{ "$ref": "#/components/schemas/SomeSchema" }] }`
#
# Runs over every `oneOf` union, wherever it is written.

walk(
  if type == "object" and (.oneOf | type) == "array"
  then .oneOf |= map(if type == "object" and has("$ref") then {allOf: [.]} else . end)
  else .
  end
)
