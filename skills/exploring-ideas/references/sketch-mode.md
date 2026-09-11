# Sketch mode: the contract with the `design` skill

Read at step 6. Two objects: the one written to `.devforgeai/explore/sketch-request.json` and passed in the Skill invocation, and the one the Design skill returns.

## Input

Read `templates/sketch-request.json` and write the request as that template with its placeholders filled, before the skill is invoked, so the request is on disk and readable whatever the skill returns.

Sixteen keys, all at the top level of one flat object, in template order and with no other key: the seven envelope keys `schema`, `id`, `phase`, `status`, `produced_by`, `consumes`, `open_questions` come first, with `status: recorded`, and then the nine fields below. The envelope sits at the top level rather than under a `meta` object — `meta` is the shape `.devforgeai/brand/tokens.json` takes, and a request written that way reaches `doc validate` as a document with no `schema`.

The nine fields:

| Field | Type | Value |
|---|---|---|
| `mode` | string | `"sketch"` |
| `idea_id` | string | `IDEA-nnn` |
| `brief_path` | string | `".devforgeai/explore/brief.md"` |
| `out_dir` | string | `".devforgeai/explore/mockups"` |
| `seed_data_path` | string | `".devforgeai/explore/seed-data.json"` |
| `fidelity` | string | `"wireframe"` |
| `flows` | array | 1 to 5 objects: `{ "flow_id": "FLOW-nnn", "name": str, "actor": str, "steps": [str], "outcome": str }` |
| `brand` | object or null | `{ "name": str, "palette_hint": str, "type_hint": str }`, or `null` when the idea has no name yet |
| `constraints` | object | `{ "no_backend": true, "no_auth": true, "no_persistence": true, "screens_per_flow_max": 3 }` |

`flows[]` holds one object per `## Core flows` row, in row order, with the same ids. `actor`, `steps`, and `outcome` come from that row; `name` is the flow's short name from `flow-drafter`. `brand.palette_hint` and `brand.type_hint` are two or three words each, drawn from `## Target user`. On a remedy run, `flows[]` holds the cited ids only.

## Invocation

```
Skill(skill="design", args="--sketch .devforgeai/explore/sketch-request.json")
```

The Design skill reads the request at that path and returns the output object below.

## Output

The Design skill writes one `FLOW-nnn-nn.html` file per screen into `out_dir`, plus `brand-sketch.json` when the request carried a `brand` object, and returns:

| Field | Type | Value |
|---|---|---|
| `mode` | string | `"sketch"` |
| `idea_id` | string | echo of the input `idea_id` |
| `screens` | array | one object per screen: `{ "flow_id": "FLOW-nnn", "screen": "FLOW-nnn-nn", "path": str, "title": str, "state": "default" \| "empty" \| "error" }` |
| `brand_sketch` | object or null | `{ "path": ".devforgeai/explore/mockups/brand-sketch.json", "name": str, "palette": [str], "type_pair": [str, str] }` |
| `uncovered_flows` | array | `FLOW-nnn` ids from the input that produced no screen; `[]` when every flow produced screens |
| `notes` | array | 0 to 5 strings, each one line, each an observation about a flow that the mockup exposed |

## Filling `## Mockups`

One row per entry of `screens[]`, in the returned order: `Flow` is `flow_id`, `Screen` is `screen`, `Path` is `path`, `State` is `state`. On a remedy run, replace the rows whose `Flow` is a cited id and leave the others as they are. Then move the brief to `status: mocked`.

`screen` is keyed `<flow id>-<two-digit screen number>` and allocates no new ID prefix. `brand_sketch` is a precursor to Design's `brand/tokens.json`, not an instance of it: sketch mode writes no `brand/tokens.json` and no `ui-specs/UI-nnn.md`, because Phase 0 has no brand kit for either to resolve against.

## Uncovered flows

Each id in `uncovered_flows` becomes one line in the brief's frontmatter `open_questions`, naming the id and the flow it belonged to. The run continues to step 7 either way; the mockup table simply carries no row for that flow.

`notes[]` entries that change a flow row belong to a later run: step 6 fills the table and does not rewrite `## Core flows`.
