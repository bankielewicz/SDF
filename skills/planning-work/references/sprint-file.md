# The sprint file

Read this before workflow step 11. It carries the eleven top-level keys of `.devforgeai/stories/sprint.yaml`, the three nested schemas, and the arithmetic `sprint-sequencer` applies. `templates/sprint.yaml` is the shape.

One file per project, rewritten by each `/plan` run. Its `id` is allocated once per epic at step 4 and reused by every later run for the same epic; a run for a different epic allocates the next id. The overwritten id survives in the frontmatter `id` of that epic's `.devforgeai/reports/SPRINT-nnn-plan.yaml`, which the ID index walks and no subcommand deletes, so `--allocate SPRINT` returns a fresh number and no report is overwritten.

## Top-level keys, in this order

| Key | Type | Required | Default | Constraint |
|---|---|---|---|---|
| `schema` | string | yes | none | constant `devforgeai/sprint/1` |
| `id` | string | yes | none | `^SPRINT-\d{3}$`, allocated by `devforgeai doc validate --allocate SPRINT` |
| `phase` | string | yes | none | constant `plan` |
| `status` | string | yes | `planned` | enum `planned`, `active`, `closed` |
| `produced_by` | string | yes | none | constant `planning-work` |
| `consumes` | list of string | yes | none | exactly one entry, the `EPIC-nnn` this sprint plans |
| `open_questions` | list of string | yes | `[]` | each one sentence, 1 to 200 characters |
| `epic` | string | yes | none | `^EPIC-\d{3}$`, equal to `consumes[0]` |
| `capacity` | object | yes | none | schema below |
| `stories` | list of object | yes | none | length `>= 1`; entry schema below |
| `deferred` | list of object | yes | `[]` | entry schema below |

Plan writes `active` at step 12, because the gate's `plan-sprint-status` check reads that value. `planned` is the enum's entry value and `closed` is written by Release. `templates/sprint.yaml` ships `active`, the value the file carries once step 12 has written it.

## `capacity`

| Field | Type | Required | Default | Constraint |
|---|---|---|---|---|
| `points_max` | integer | yes | `20` | `>= 1`; the value of `config.toml` `[plan].sprint_capacity_points` when that key is present |
| `points_planned` | integer | yes | none | the sum of `stories[].points` |
| `source` | string | yes | none | enum `config` (the key was present), `default` (the key was absent and 20 was used) |

## `stories[]`

| Field | Type | Required | Default | Constraint |
|---|---|---|---|---|
| `id` | string | yes | none | `^STORY-\d{3}$` resolving to a `stories/STORY-nnn.md` file; first key in the entry |
| `status` | string | yes | none | a member of the story `status` enum; second key in the entry |
| `order` | integer | yes | none | `1` to `len(stories)`, unique, dense; a topological order of the dependency graph |
| `points` | integer | yes | none | a member of `config.toml` `[plan].story_points`, default set `[1, 2, 3, 5, 8]` |

`id` precedes `status` in every entry, which the `pre-push` hook relies on.

## `deferred[]`

| Field | Type | Required | Default | Constraint |
|---|---|---|---|---|
| `id` | string | yes | none | `^STORY-\d{3}$` resolving to a `stories/STORY-nnn.md` file, absent from `stories[]` |
| `reason` | string | yes | none | enum `capacity` (adding it exceeds `points_max`), `dependency` (a story it depends on is deferred) |

The union of `stories[].id` and `deferred[].id` is every story written for the epic, so `devforgeai story validate --scope sprint` reaches all of them. Every story of the epic appears exactly once across the two lists.

## The capacity arithmetic

`points_max` is `config.toml` `[plan].sprint_capacity_points`, and `20` when that key is absent; `capacity.source` records which of the two applied, so the file's own history says where the number came from.

`sprint-sequencer` walks the topological order and appends a story while `points_planned + points <= points_max`, so a story landing exactly on `points_max` enters the sprint. A story that would take the sum past `points_max` is deferred with `reason: capacity`, and so is every story downstream of it in the dependency graph, with `reason: dependency` — a story whose input is not being built this sprint has nothing to build against.

A story whose own `points` alone exceed `points_max` enters the sprint as the sole entry of `stories[]`, `points_planned` equals its `points`, and every other story is deferred. An empty `stories[]` would leave the Build phase with no subject and fail the gate's `plan-docs` check, so one oversized story is the better of the two outcomes.

`longest_chain` comes back alongside these lists: the longest dependency chain among the ids in `stories[]`. It is reported and read by no check. A non-empty `cycle` holds one cycle's id path and stops the run before `sprint.yaml` is written.
