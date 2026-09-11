# The key namespace

Read at workflow steps 4, 5, 6, and 8, which write the `| Key | Value | Source |`
tables.

A table whose header row is exactly `| Key | Value | Source |` contributes
`(key, value, file, line)` triples to one namespace shared across the six files.
`devforgeai context audit` check CA-5 collects every such triple and fails when
one key holds two distinct values, or when a key falls outside the closed set
below. Any other table shape — `| Rule | Scope | Source |`, `| Layer | Purpose |`,
`| Technology | Reason | Recorded in |` — is prose to CA-5 and contributes
nothing.

That is why the same key repeated in a second file with the same value passes:
`tokens.path` appears in coding-standards.md and nowhere else, but
`language.primary` restated in dependencies.md with the same value is a
duplicate the audit allows, and restated with a different value is the
contradiction the audit exists to find.

## The closed set

| Key | Value domain | Owning file |
|---|---|---|
| `language.primary` | identifier | tech-stack.md |
| `language.primary.version` | version string | tech-stack.md |
| `language.secondary.<n>` | identifier, `<n>` integer | tech-stack.md |
| `language.secondary.<n>.version` | version string | tech-stack.md |
| `runtime.name` | identifier | tech-stack.md |
| `runtime.version` | version string | tech-stack.md |
| `framework.<role>` | identifier; `<role>` in `web api ui orm test build lint format package` | tech-stack.md |
| `framework.<role>.version` | version string | tech-stack.md |
| `datastore.<role>` | identifier; `<role>` in `primary cache search queue blob` | tech-stack.md |
| `datastore.<role>.version` | version string | tech-stack.md |
| `source.root` | path relative to project root | source-tree.md |
| `test.root` | path relative to project root | source-tree.md |
| `build.output.root` | path relative to project root | source-tree.md |
| `layer.<name>.path` | glob | source-tree.md |
| `dep.<name>.version` | version constraint string | dependencies.md |
| `dep.<name>.scope` | one of `runtime dev test build` | dependencies.md |
| `style.indent` | integer | coding-standards.md |
| `style.line.max` | integer | coding-standards.md |
| `style.quote` | one of `single double` | coding-standards.md |
| `naming.<entity>` | one of `camel pascal snake kebab upper-snake`; `<entity>` in `type function variable constant` | coding-standards.md |
| `naming.<entity>` | one of `camel pascal snake kebab upper-snake`; `<entity>` in `file directory test-file` | source-tree.md |
| `tokens.path` | path relative to project root | coding-standards.md |

## How a concrete key matches a wildcard row

Matching is by dot-separated segment. `<name>`, `<entity>`, and `<role>` each
match exactly one dot-free segment, drawn from the row's value set where the row
states one. `<n>` matches one to three decimal digits.

| Concrete key | Matches | Why |
|---|---|---|
| `layer.domain.path` | `layer.<name>.path` | one segment between `layer` and `path` |
| `dep.serde.version` | `dep.<name>.version` | one segment for `<name>` |
| `framework.orm` | `framework.<role>` | `orm` is in the role list |
| `layer.domain.core.path` | nothing | two segments where the row allows one |
| `framework.cli` | nothing | `cli` is outside the role list |

A key outside the namespace fails CA-5 with the same code as a key holding two
values, so an invented key and a contradiction surface the same way.

The literal wildcard strings — `framework.<role>`, `layer.<name>.path`,
`dep.<name>.version` — stand in the templates as placeholder rows. A file at
`status: accepted` carries concrete keys in their place: CA-2 requires the
accepted status, CA-5 rejects the literal `<role>` as a key outside the
namespace, and the two together keep a template row from shipping as content.

## What is not a key

Constraint status and anti-pattern severity are not in this namespace. Each
lives once, in `## Constraint index` and `## Anti-pattern index`, and the audit
reads them there through checks CA-4, CA-6, and CA-8. Writing `con.003.status`
into a `| Key | Value | Source |` table puts the same fact in two places and
fails CA-5 as a key outside the set.

## The `Source` column

The third cell records where the value came from, so a later run can tell a
detected fact from a decided one.

| Value came from | `Source` cell |
|---|---|
| `devforgeai stack detect` | `config.toml` |
| a `## Technology scan` row with `Maturity: established` | `explore/brief.md ## Technology scan` |
| a decision this phase records | the `ADR-nnn` that records it |
| a brownfield draft | `init --analyze` |
| the `source-tree-mapper` output | `source-tree-mapper` |
| Design's token file | `Design` |
