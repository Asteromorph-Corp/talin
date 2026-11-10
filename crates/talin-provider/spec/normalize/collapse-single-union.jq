# Collapses a single-armed `oneOf` union to its content.
#
# `{ "oneOf": [ { "type": "string" } ] }` -> `{ "type": "string" }`

walk(
  if type == "object" and (.oneOf | type) == "array" and (.oneOf | length) == 1
  then
    # The arm wins every key it defines, so that what the wrapper adds — a `default` beside a
    # `$ref`, say — is carried over rather than overwriting the schema it wraps.
    # The exception is `title`: a wrapper's title names the component it came from, which is what
    # `oasdiff` pairs union arms by, so the arm's own title must not replace it.
    .oneOf[0] as $arm
    | (del(.oneOf) | del(.description)) as $rest
    | $rest + $arm
      + (if has("title") then {title} else {} end)
      + (if ($arm | has("description")) or (has("description") | not)
         then {} else {description: .description} end)
  else . end
)
