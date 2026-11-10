#!/usr/bin/env python3

"""Rewrites the API reference page into an OpenAPI description.

The API publishes its reference as Markdown rather than as a machine-readable description, so this
reads that page and writes what it describes as OpenAPI 3.1. It is the fetch step's second half:
`sync.sh` retrieves the page and pipes it through here.

The page is regular enough to parse without a Markdown library. Every parameter, field and type is
a bullet whose entire content is one backticked declaration, and the nesting of the bullets is the
nesting of the types:

    - `CacheControlEphemeral object { type, ttl }`

      - `type: "ephemeral"`

        - `"ephemeral"`

      - `ttl: optional "5m" or "1h"`

        The time-to-live for the cache control breakpoint.

A declaration names its own type, so a nested bullet that merely spells out a type named further up
is redundant and this drops it. What it does read the nested bullets for is what a declaration
leaves unsaid: the arms of a union, which the declaration may elide as `or 3 more` and may leave
ambiguous as `array of A or B`, and the shape of an anonymous type.

Reads the page on stdin or from the path given as an argument, and writes the description on
stdout.
"""

import json
import re
import sys
from dataclasses import dataclass, field
from typing import cast

Schema = dict[str, object]

# The page states these in prose and in its example requests rather than in a declaration, so they
# are written here.
SERVERS: list[Schema] = [{"url": "https://api.anthropic.com", "description": "Production Server"}]
SECURITY_SCHEMES: dict[str, Schema] = {
    "apiKeyAuth": {
        "type": "apiKey",
        "in": "header",
        "name": "x-api-key",
        "description": "An Anthropic API key sent as the `x-api-key` header.",
    }
}
SECURITY: list[Schema] = [{"apiKeyAuth": []}]
VERSION_PARAMETER: Schema = {
    "name": "anthropic-version",
    "in": "header",
    "required": True,
    "description": "The version of the Anthropic API to use (e.g. 2023-06-01).",
    "schema": {"type": "string"},
}
# Every response is JSON except the batch results, which the page describes as a `.jsonl` file.
RESPONSE_CONTENT_TYPES: dict[str, str] = {
    "/v1/messages/batches/{message_batch_id}/results": "application/x-jsonl",
}
# The types the page lists under `Returns` that are a stream of events rather than a body, and the
# content type a stream of them is served as.
STREAMED: set[str] = {"RawMessageStreamEvent"}
STREAM_CONTENT_TYPE = "text/event-stream"


HEADING = re.compile(r"^(?P<hashes>#{1,6}) +(?P<title>.+?) *$")
BULLET = re.compile(r"^(?P<indent> *)- `(?P<code>.+)`$")
FENCE = re.compile(r"^(?P<indent> *)```")
# `**POST** `/v1/messages``, under the heading that names the operation.
OPERATION = re.compile(r"^\*\*(?P<method>[A-Za-z]+)\*\* `(?P<path>/[^`]*)`$")

# `max_tokens: number`, or `"anthropic-user-profile-id": optional string` for a header.
FIELD = re.compile(r'^"?(?P<name>[a-z_][A-Za-z0-9_.-]*)"?: (?P<type>.+)$')
# `Base64ImageSource object`, or `Base64ImageSource object { data, media_type, type }` as the
# page used to write it.
NAMED_OBJECT = re.compile(r"^(?P<name>[A-Z][A-Za-z0-9_]*) object( \{[^}]*\})?$")
# `RawContentBlockDelta = TextDelta or InputJSONDelta or 3 more`.
NAMED_ALIAS = re.compile(r"^(?P<name>[A-Z][A-Za-z0-9_]*) = (?P<type>.+)$")
# The tail of a union the page elided, as in `or 3 more`.
ELISION = re.compile(r"^\d+ more$")
NAME = re.compile(r"^[A-Z][A-Za-z0-9_]*$")

# The headings an operation is read under. Matched case-folded, because the page restyles them:
# `Body Parameters` became `Body parameters` without anything under it changing. `Headers` was
# `Header Parameters` under the same restyle, and both spellings are read, so settling back on
# either one still converts.
PARAMETER_HEADINGS: dict[str, str] = {
    "headers": "header",
    "header parameters": "header",
    "path parameters": "path",
    "query parameters": "query",
}
BODY_HEADING = "body parameters"
RETURNS_HEADING = "returns"

SCALARS: dict[str, str] = {"string": "string", "number": "number", "boolean": "boolean"}


def group(match: re.Match[str], name: str) -> str:
    """The named group, which every pattern here defines and so always matches to a string."""

    return str(match.group(name))


class Bullet:
    """One backticked declaration, the prose under it, and the bullets nested inside it."""

    def __init__(self, code: str) -> None:
        self.code: str = code
        self.lines: list[str] = []
        self.children: list[Bullet] = []

    @property
    def description(self) -> str:
        return "\n".join(self.lines).strip()


class Section:
    """One heading, the prose under it, and the bullets under it."""

    def __init__(self, level: int, title: str) -> None:
        self.level: int = level
        self.title: str = title
        self.lines: list[str] = []
        self.bullets: list[Bullet] = []

    @property
    def description(self) -> str:
        return "\n".join(self.lines).strip()


@dataclass
class Operation:
    """One operation, filled in from the sections under the heading that names it."""

    path: str
    method: str
    tag: str
    title: str
    description: str
    parameters: list[Schema] = field(default_factory=list)
    request_body: Schema | None = None
    responses: Schema = field(default_factory=dict)

    def document(self) -> Schema:
        document: Schema = {
            "operationId": operation_id(self.title),
            "summary": self.title,
            "description": self.description,
            "tags": [self.tag] if self.tag else [],
            "security": SECURITY,
            "parameters": self.parameters,
            "responses": self.responses,
        }
        if self.request_body is not None:
            document["requestBody"] = self.request_body
        return document


def parse(text: str) -> list[Section]:
    """Reads the page into its sections, each holding a tree of bullets."""

    sections: list[Section] = [Section(0, "")]
    # The bullets currently open, innermost last, as (indent, bullet).
    stack: list[tuple[int, Bullet]] = []
    # The indent the open fence was opened at, or None outside a fence.
    fence: int | None = None

    def place(line: str, indent: int) -> None:
        """Files a prose line under the innermost bullet it is indented inside of."""

        while stack and indent <= stack[-1][0]:
            del stack[-1]
        (stack[-1][1] if stack else sections[-1]).lines.append(line)

    for raw in text.split("\n"):
        line = raw.rstrip()

        opening = FENCE.match(line)
        if fence is not None:
            # A fenced block is prose; keeping it means keeping its own indentation.
            place(line[fence:], fence + 1)
            if opening:
                fence = None
            continue
        if opening:
            fence = len(group(opening, "indent"))
            place(line[fence:], fence + 1)
            continue

        heading = HEADING.match(line)
        if heading:
            stack.clear()
            sections.append(Section(len(group(heading, "hashes")), group(heading, "title")))
            continue

        bullet = BULLET.match(line)
        if bullet:
            indent = len(group(bullet, "indent"))
            while stack and indent <= stack[-1][0]:
                del stack[-1]
            node = Bullet(group(bullet, "code"))
            (stack[-1][1].children if stack else sections[-1].bullets).append(node)
            stack.append((indent, node))
            continue

        # A prose line, or a blank one; blanks are kept so paragraphs survive the join.
        place(line.strip(), len(line) - len(line.lstrip()) if line.strip() else 1 << 30)

    return sections[1:]


def split_union(declaration: str) -> list[str]:
    """Splits `A or B or C` into its arms, leaving `or` inside a string or a brace list alone."""

    arms: list[str] = []
    depth = 0
    quoted = False
    arm = ""
    rest = declaration
    while rest:
        if not quoted and depth == 0 and rest.startswith(" or "):
            arms.append(arm)
            arm = ""
            rest = rest[4:]
            continue
        if rest[0] == '"':
            quoted = not quoted
        elif not quoted and rest[0] in "{[":
            depth += 1
        elif not quoted and rest[0] in "}]":
            depth -= 1
        arm += rest[0]
        rest = rest[1:]
    arms.append(arm)
    return arms


def described(schema: Schema, description: str) -> Schema:
    if description:
        schema["description"] = description
    return schema


def nullable(schema: Schema) -> Schema:
    """Admits null, the way the crate's own description spells it."""

    kind = schema.get("type")
    if isinstance(kind, str) and "enum" not in schema and "$ref" not in schema:
        return {**schema, "type": [kind, "null"]}
    return {"oneOf": [{"type": "null"}, schema]}


def streams(bullet: Bullet) -> bool:
    """Whether a `Returns` bullet declares the stream of events rather than the body."""

    named = NAMED_OBJECT.match(bullet.code) or NAMED_ALIAS.match(bullet.code)
    return named is not None and group(named, "name") in STREAMED


def spells_arms(bullets: list[Bullet]) -> bool:
    """Whether a list of bullets spells out the arms of a union rather than an object's fields."""

    return bool(bullets) and not any(FIELD.match(bullet.code) for bullet in bullets)


def arm_bullets(bullet: Bullet) -> list[Bullet]:
    """The bullets under a union that spell out its arms, rather than an object's fields."""

    return bullet.children if spells_arms(bullet.children) else []


def string_enum(values: list[str]) -> Schema:
    return {"type": "string", "enum": values}


def enum_values(schema: Schema) -> list[str] | None:
    """The values of a plain string enum, or None for a schema that is anything else."""

    values = schema.get("enum")
    if list(schema) != ["type", "enum"] or schema["type"] != "string":
        return None
    if not isinstance(values, list):
        return None
    return [value for value in cast(list[object], values) if isinstance(value, str)]


def folded_null(schema: Schema) -> Schema | None:
    """`oneOf[null, a string enum]` written as one nullable enum, or None for anything else."""

    wrapped = schema.get("oneOf")
    if not isinstance(wrapped, list):
        return None
    arms = cast(list[object], wrapped)
    if len(arms) != 2 or arms[0] != {"type": "null"} or not isinstance(arms[1], dict):
        return None
    values = enum_values(cast(Schema, arms[1]))
    if values is None:
        return None
    rest = {key: value for key, value in schema.items() if key != "oneOf"}
    return {**rest, "type": ["string", "null"], "enum": values}


def pascal(name: str) -> str:
    """`content_block` becomes `ContentBlock`, so a minted name reads as one word per word."""

    return "".join(word[:1].upper() + word[1:] for word in name.split("_"))


class Converter:
    def __init__(self) -> None:
        # The names the page defines (see `collect`). A declaration naming one of
        # these is a reference to it, whatever the bullets nested under the declaration spell out.
        self.components: set[str] = set()
        self.definitions: dict[str, Bullet] = {}
        self.warnings: list[str] = []

    def warn(self, message: str) -> None:
        self.warnings.append(message)

    # -- Declarations --------------------------------------------------------------------------

    def schema(self, declaration: str, bullet: Bullet) -> Schema:
        """Converts one type declaration, reading `bullet`'s children for what it leaves out."""

        declaration = declaration.strip()

        alias = NAMED_ALIAS.match(declaration)
        if alias:
            name, definition = group(alias, "name"), group(alias, "type")
            # The page labels an arm with the name of the type it is made of rather than the name
            # of its own shape, so `X = array of X` is an array of X and not X itself. The label is
            # a reference to the type only where the definition does not name it again.
            if name in self.components and not re.search(rf"\b{re.escape(name)}\b", definition):
                return self.reference(name)
            declaration = definition

        arms = split_union(declaration)
        arms[0] = arms[0].removeprefix("optional ")
        admits_null = "null" in arms
        arms = [arm for arm in arms if arm != "null"]

        schema = self.arms(arms, bullet)
        return nullable(schema) if admits_null else schema

    def arms(self, arms: list[str], bullet: Bullet) -> Schema:
        """Converts the arms of one union, which the bullets under it have the last word on."""

        if len(arms) == 1:
            return self.term(arms[0], bullet)

        spelled = arm_bullets(bullet)
        # `array of A or B` is written the same whether the array covers the whole union or only
        # its first arm. What the page spells out underneath settles it: an expansion that names
        # the array again is a union of arms, and one that does not is the array's item type.
        if arms[0].startswith("array of ") and not any("array of " in arm.code for arm in spelled):
            items = [arms[0][len("array of ") :], *arms[1:]]
            return {"type": "array", "items": self.union(self.arm_schemas(items, spelled, bullet))}

        return self.union(self.arm_schemas(arms, spelled, bullet))

    def arm_schemas(self, arms: list[str], spelled: list[Bullet], bullet: Bullet) -> list[Schema]:
        """One schema per arm, off the bullets that spell them out where the page wrote them."""

        if spelled:
            return [self.schema(arm.code, arm) for arm in spelled]
        if any(ELISION.match(arm) for arm in arms):
            self.warn(f"`{' or '.join(arms)}` is elided with nothing under it to spell it out")
        return [self.term(arm, bullet) for arm in arms]

    def term(self, term: str, bullet: Bullet) -> Schema:
        """Converts one arm of a union, which is where every actual type is named."""

        if term.startswith("array of "):
            return {"type": "array", "items": self.schema(term[len("array of ") :], bullet)}

        if term.startswith('"') and term.endswith('"'):
            return string_enum([term[1:-1]])

        if term in SCALARS:
            return {"type": SCALARS[term]}
        if term == "map[unknown]":
            return {"type": "object", "additionalProperties": True}
        if term == "unknown":
            return {}

        named = NAMED_OBJECT.match(term)
        if named:
            if group(named, "name") in self.components:
                return self.reference(group(named, "name"))
            return self.object(bullet)
        if term == "object" or term.startswith("object {"):
            return self.object(bullet)

        alias = NAMED_ALIAS.match(term)
        if alias:
            if group(alias, "name") in self.components:
                return self.reference(group(alias, "name"))
            return self.schema(group(alias, "type"), bullet)

        if NAME.match(term):
            if term in self.components:
                return self.reference(term)
            if bullet.children:
                return self.spelled(bullet.children)
            self.warn(f"no type named {term} is defined")
            return {}

        self.warn(f"cannot read the type `{term}`")
        return {}

    def union(self, arms: list[Schema]) -> Schema:
        """Collects the arms, folding every arm that is a single value of a string enum into one."""

        if len(arms) == 1:
            return arms[0]

        merged: list[Schema] = []
        values: list[str] = []
        position: int | None = None
        for arm in arms:
            arm_values = enum_values(arm)
            if arm_values is None:
                merged.append(arm)
                continue
            if position is None:
                position = len(merged)
            values += arm_values

        if position is None:
            return {"oneOf": arms}
        merged.insert(position, string_enum(values))
        return merged[0] if len(merged) == 1 else {"oneOf": merged}

    def object(self, bullet: Bullet) -> Schema:
        return self.fields(bullet.children, {"type": "object"})

    def spelled(self, bullets: list[Bullet]) -> Schema:
        """The shape a list of bullets spells out, as the arms of a union or as an object."""

        if spells_arms(bullets):
            return self.union([self.schema(bullet.code, bullet) for bullet in bullets])
        return self.fields(bullets, {"type": "object"})

    def fields(self, bullets: list[Bullet], schema: Schema) -> Schema:
        """Converts a list of field declarations into an object schema."""

        properties: dict[str, Schema] = {}
        required: list[str] = []
        for bullet in bullets:
            match = FIELD.match(bullet.code)
            if not match:
                self.warn(f"cannot read the field `{bullet.code}`")
                continue
            name, declaration = group(match, "name"), group(match, "type")
            properties[name] = described(self.schema(declaration, bullet), bullet.description)
            if not declaration.startswith("optional "):
                required.append(name)

        schema["properties"] = properties
        if required:
            schema["required"] = required
        return schema

    def reference(self, name: str) -> Schema:
        return {"$ref": f"#/components/schemas/{name}"}

    # -- Sections ------------------------------------------------------------------------------

    def collect(self, sections: list[Section]) -> None:
        """Records the types the page defines, before anything reads a declaration.

        The page defines a type where it first spells one out: a named object or alias, or a field
        whose whole type is a name, with the arms or fields of that type nested under it. Every
        later mention of the name is a reference, spelled out again or not.
        """

        def visit(bullets: list[Bullet]) -> None:
            for bullet in bullets:
                named = self.defined(bullet)
                if named is not None and named not in self.definitions and bullet.children:
                    self.definitions[named] = bullet
                visit(bullet.children)

        for section in sections:
            visit(section.bullets)
        self.components = set(self.definitions)

    def defined(self, bullet: Bullet) -> str | None:
        """The name of the type a bullet defines, or None for one that only uses a type."""

        named = NAMED_OBJECT.match(bullet.code) or NAMED_ALIAS.match(bullet.code)
        if named:
            return group(named, "name")

        field_match = FIELD.match(bullet.code)
        if not field_match:
            return None
        arms = [
            arm
            for arm in split_union(group(field_match, "type").removeprefix("optional "))
            if arm != "null"
        ]
        if len(arms) != 1:
            return None
        term = arms[0].removeprefix("array of ")
        return term if NAME.match(term) else None

    def schemas(self, sections: list[Section]) -> dict[str, Schema]:
        schemas: dict[str, Schema] = {}
        for name, bullet in self.definitions.items():
            if NAMED_OBJECT.match(bullet.code):
                schema = described(self.object(bullet), bullet.description)
            elif alias := NAMED_ALIAS.match(bullet.code):
                definition = group(alias, "type")
                if definition == f"array of {name}" and bullet.children:
                    # The page labels the array with the name of its item type, and spells that
                    # type's arms out underneath. Fields that name it say `array of X`.
                    schema = described(self.spelled(bullet.children), bullet.description)
                else:
                    schema = described(self.schema(definition, bullet), bullet.description)
            else:
                # A field that names its own type; the prose under it describes the field.
                schema = self.spelled(bullet.children)
            schemas[name] = schema
        return schemas

    def parameters(self, section: Section, location: str) -> list[Schema]:
        parameters: list[Schema] = []
        for bullet in section.bullets:
            match = FIELD.match(bullet.code)
            if not match:
                self.warn(f"cannot read the {location} parameter `{bullet.code}`")
                continue
            declaration = group(match, "type")
            parameters.append(
                {
                    "name": group(match, "name"),
                    "in": location,
                    # A path parameter is part of the path, so it is required whatever it says.
                    "required": location == "path" or not declaration.startswith("optional "),
                    "description": bullet.description,
                    "schema": self.schema(declaration, bullet),
                }
            )
        return parameters

    def returns(self, section: Section, path: str) -> Schema:
        """The response body, and the stream of events the page lists beside it where there is one."""

        # `Create a Message` lists two shapes under the one heading: the message a request gets,
        # and the stream of events a request that asked to stream gets instead. Those are two ways
        # of serving the one response rather than two shapes of one body, so each is keyed by the
        # content type it is served as instead of being folded into a union with the other.
        bodies = [bullet for bullet in section.bullets if not streams(bullet)]
        streamed = [bullet for bullet in section.bullets if streams(bullet)]

        content: Schema = {}
        if bodies:
            content[RESPONSE_CONTENT_TYPES.get(path, "application/json")] = {
                "schema": self.spelled(bodies)
            }
        if streamed:
            content[STREAM_CONTENT_TYPE] = {"schema": self.spelled(streamed)}

        return {
            "200": {
                "description": section.description or "Successful response",
                "content": content,
            }
        }

    def paths(self, sections: list[Section]) -> dict[str, Schema]:
        operations: list[Operation] = []
        tag = ""
        operation: Operation | None = None
        depth = 0

        for section in sections:
            match = OPERATION.match(section.description.split("\n")[0])
            if match:
                operation = Operation(
                    path=group(match, "path"),
                    # OpenAPI keys an operation by a lowercase method, whichever case the page used.
                    method=group(match, "method").lower(),
                    tag=tag,
                    title=section.title,
                    description="\n".join(section.description.split("\n")[1:]).strip(),
                    parameters=[dict(VERSION_PARAMETER)],
                )
                depth = section.level
                operations.append(operation)
                continue

            if section.level == 1:
                # A page that groups its operations names the group in a heading of its own.
                tag = section.title
                operation = None
                continue

            # A heading at or above the open operation's own ends it, whether it opens the domain
            # types, another operation, or a group of them. Anything nested deeper belongs to it.
            if operation is not None and section.level <= depth:
                operation = None
            if operation is None:
                continue

            title = section.title.casefold()
            if title in PARAMETER_HEADINGS:
                # The page now lists `anthropic-version` as optional, because its SDKs fill it in.
                # The API requires it, so the entry written here stands over the page's.
                known = {parameter["name"] for parameter in operation.parameters}
                operation.parameters += [
                    parameter
                    for parameter in self.parameters(section, PARAMETER_HEADINGS[title])
                    if parameter["name"] not in known
                ]
            elif title == BODY_HEADING:
                operation.request_body = {
                    "required": True,
                    "content": {
                        "application/json": {
                            "schema": self.fields(section.bullets, {"type": "object"})
                        }
                    },
                }
            elif title == RETURNS_HEADING:
                operation.responses = self.returns(section, operation.path)

        paths: dict[str, Schema] = {}
        for operation in operations:
            methods: Schema = paths.setdefault(operation.path, {})
            methods[operation.method] = operation.document()
        return paths

    def fold_tags(self, schemas: dict[str, Schema]) -> None:
        """Rewrites the tag property of each arm of a tagged union as a nullable enum.

        `spec/normalize/title-arm-by-tag.jq` finds a union's tag by the string each arm pins it to,
        and cannot see a pin wrapped in a `oneOf`, which is how the page's `optional "custom" or
        null` is spelled. A tag is a property that at least two arms of a union of references fold
        to a nullable enum, no two to the same value; it is folded in every arm that can be.
        """

        for arms in (schema.get("oneOf") for schema in schemas.values()):
            if not isinstance(arms, list):
                continue
            # The properties of each arm, as the arm's component would hold them.
            members: list[dict[str, object]] = []
            for arm in cast(list[object], arms):
                reference = cast(Schema, arm).get("$ref") if isinstance(arm, dict) else None
                if not isinstance(reference, str):
                    break
                properties = schemas.get(reference.split("/")[-1], {}).get("properties")
                if not isinstance(properties, dict):
                    break
                members.append(cast(dict[str, object], properties))
            else:
                names = {name for properties in members for name in properties}
                for name in sorted(names):
                    folds = [
                        folded_null(cast(Schema, p[name])) if isinstance(p.get(name), dict) else None
                        for p in members
                    ]
                    # An arm that pins the tag without admitting null counts as a pin, too.
                    values = [
                        tuple(cast(list[str], f["enum"])) if f is not None else tuple(plain)
                        for f, p in zip(folds, members)
                        if f is not None
                        or (
                            isinstance(p.get(name), dict)
                            and (plain := enum_values(cast(Schema, p[name])))
                        )
                    ]
                    if not any(folds) or len(values) < 2 or len(set(values)) != len(values):
                        continue
                    for properties, folded in zip(members, folds):
                        if folded is not None:
                            properties[name] = folded

    def inline_unions(self, schemas: dict[str, Schema]) -> None:
        """Lifts each union the page writes inline into a component of its own."""

        defined = dict(schemas)

        def referenced(arms: object) -> list[object] | None:
            """The arms, for a union whose every arm is a reference to a named type."""

            if not isinstance(arms, list) or len(cast(list[object], arms)) < 2:
                return None
            for arm in cast(list[object], arms):
                if not isinstance(arm, dict) or list(cast(Schema, arm)) != ["$ref"]:
                    return None
            return cast(list[object], arms)

        def named(arms: list[object], owner: str, field: str) -> str | None:
            """The name to reference the union by, or None where one cannot be given."""

            for name in sorted(defined):
                if defined[name].get("oneOf") == arms:
                    return name

            name = owner + pascal(field)
            if name in schemas and schemas[name].get("oneOf") != arms:
                self.warn(f"{owner}.{field} cannot be lifted: {name} is already something else")
                return None
            schemas[name] = {"oneOf": arms}
            return name

        def lift(node: Schema, owner: str, field: str) -> bool:
            arms = referenced(node.get("oneOf"))
            if arms is None:
                return False
            name = named(arms, owner, field)
            if name is None:
                return False
            del node["oneOf"]
            node["$ref"] = f"#/components/schemas/{name}"
            return True

        def visit(node: object, owner: str, field: str | None) -> None:
            if isinstance(node, list):
                for item in cast(list[object], node):
                    visit(item, owner, field)
                return
            if not isinstance(node, dict):
                return
            if field is not None and lift(cast(Schema, node), owner, field):
                return
            for key, value in cast(Schema, node).items():
                if key == "properties" and isinstance(value, dict):
                    for name, schema in cast(Schema, value).items():
                        visit(schema, owner, name)
                else:
                    visit(value, owner, field)

        for owner, schema in list(schemas.items()):
            visit(schema, owner, None)

    def convert(self, text: str) -> Schema:
        sections = parse(text)
        self.collect(sections)

        schemas = self.schemas(sections)
        self.inline_unions(schemas)
        self.fold_tags(schemas)

        title = next((section.title for section in sections if section.level == 1), "API")
        return {
            "openapi": "3.1.0",
            "info": {
                "title": title,
                # The page carries no version of its own; this one names what it is instead.
                "version": "reference",
                "description": (
                    "OpenAPI description of the Claude API, as its reference page documents it. "
                    "Converted by spec/claude_messages/from-markdown.py."
                ),
            },
            "servers": SERVERS,
            "paths": self.paths(sections),
            "components": {
                "schemas": schemas,
                "securitySchemes": SECURITY_SCHEMES,
            },
        }


def operation_id(title: str) -> str:
    """`Create a Message` becomes `createMessage`, which is the name the API gives it."""

    words = [match.group(0) for match in re.finditer(r"[A-Za-z0-9]+", title)]
    words = [word for word in words if word.lower() not in {"a", "an", "the"}]
    return "".join(
        word.lower() if index == 0 else word[0].upper() + word[1:]
        for index, word in enumerate(words)
    )


def main() -> None:
    if len(sys.argv) > 1:
        with open(sys.argv[1]) as page:
            text = page.read()
    else:
        text = sys.stdin.read()

    converter = Converter()
    description = converter.convert(text)

    for warning in dict.fromkeys(converter.warnings):
        print(f"from-markdown.py: {warning}", file=sys.stderr)

    # Sorted, to match what the rest of spec/ commits.
    json.dump(description, sys.stdout, indent=2, sort_keys=True)
    print()


if __name__ == "__main__":
    main()
