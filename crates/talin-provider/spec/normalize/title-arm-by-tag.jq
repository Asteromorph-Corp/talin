# Titles an arm of a `oneOf` union with the value of the tag property in that arm.
#
# `{ "properties": { "type": { "const": "foo" } } }` ->
# `{ "properties": { "type": { "const": "foo" } }, "title": "foo" }`
#
# `{ "properties": { "type": { "enum": ["foo"] } } }` ->
# `{ "properties": { "type": { "enum": ["foo"] } }, "title": "foo" }`
#
# An arm whose tag accepts several values is titled with the alphabetically smallest one.
#
# `{ "properties": { "type": { "enum": ["foo_20260101", "foo"] } } }` ->
# `{ "properties": { "type": { "enum": ["foo_20260101", "foo"] } }, "title": "foo" }`
#
# Runs over every `oneOf` union, wherever it is written. The tag of a union is the property that
# the most of its arms pin to a string, provided at least two arms pin it and no two pin the same
# value. A tie goes to the property that comes first alphabetically. Arms that do not pin the tag
# stay untitled, and a union with no such property is left alone.

def pinned:
  [
    (.const | select(type == "string")),
    (.enum | select(type == "array") | map(select(type == "string")) | select(length > 0) | min)
  ]
  | first;

# The string-pinned properties of an arm, as property name to pinned value.
def pins:
  if type != "object" then {}
  else
    (.properties? // {})
    | with_entries(select(.value | type == "object") | .value |= pinned)
    | with_entries(select(.value | type == "string"))
  end;

# The tag of a union, given the pins of its arms.
def tag_of:
  . as $pins
  | [
      ($pins | map(keys) | add // [] | unique[]) as $key
      | [$pins[] | select(has($key)) | .[$key]] as $values
      | select(($values | length) >= 2 and ($values | unique | length) == ($values | length))
      | {key: $key, count: ($values | length)}
    ]
  | sort_by(-.count, .key)
  | first
  | .key?;

walk(
  if type == "object" and (.oneOf | type) == "array"
  then
    (.oneOf | map(pins) | tag_of) as $tag
    | if $tag == null then .
      else .oneOf |= map(if (pins | has($tag)) then .title = pins[$tag] else . end)
      end
  else .
  end
)
