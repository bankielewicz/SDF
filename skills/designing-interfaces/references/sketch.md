# Sketch mode

Read before workflow step 2. Sketch mode is the mode Explore step 6 calls. It draws wireframe screens for a set of `FLOW-nnn` flows and returns JSON. It writes no `brand/tokens.json` and no `ui-specs/UI-nnn.md`, and it emits no send-back.

## The request

Explore writes `.devforgeai/explore/sketch-request.json` before invoking this skill, so the request is on disk and readable independently of the invocation. It carries the seven document keys — `schema`, `id`, `phase`, `status`, `produced_by`, `consumes`, `open_questions` — followed by these fields:

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

An absent request file is the one failure path of step 2: return `screens: []`, `uncovered_flows` holding every `flow_id` the invocation passed, and one `notes` line naming the missing path.

## The return value

Returned as JSON, with the referenced files written to `out_dir`:

| Field | Type | Value |
|---|---|---|
| `mode` | string | `"sketch"` |
| `idea_id` | string | echo of the input `idea_id` |
| `screens` | array | one object per screen: `{ "flow_id": "FLOW-nnn", "screen": "FLOW-nnn-nn", "path": str, "title": str, "state": "default" \| "empty" \| "error" }` |
| `brand_sketch` | object or null | `{ "path": ".devforgeai/explore/mockups/brand-sketch.json", "name": str, "palette": [str], "type_pair": [str, str] }` |
| `uncovered_flows` | array | `FLOW-nnn` ids from the input that produced no screen; `[]` when every flow produced screens |
| `notes` | array | 0 to 5 strings, each one line, each an observation about a flow that the mockup exposed |

Explore fills the brief's `## Mockups` table from `screens`, and adds one `open_questions` line per id in `uncovered_flows`.

## Screen naming

`screen` is `<flow id>-<two-digit screen number>`, so `FLOW-002-01` is the first screen of `FLOW-002`. Numbering starts at `01` per flow and runs in the order the flow's `steps` run. The screen id allocates no new ID prefix; `doc validate` does not index it.

The file for a screen is `<out_dir>/<screen>.html`, so `FLOW-002-01` lands at `.devforgeai/explore/mockups/FLOW-002-01.html`. `screens[].path` carries that path.

`constraints.screens_per_flow_max` caps the screens drawn per flow. A flow whose steps produce no screen at all comes back in `uncovered_flows`; the remaining flows keep their screens.

## The wireframe files

`templates/sketch-screen.html` is the shape. One file per screen, static HTML with a `<style>` element: no script, no build step, no dependency manifest, no framework. The seed rows come from the `entities[].rows` arrays of `.devforgeai/explore/seed-data.json`, so the screen shows the data the flow actually moves rather than lorem text.

The header line carries the flow id, the screen number, and the state; the main region carries the regions the flow step needs; the footer names the next screen or the outcome the flow names. `state` is one of `default`, `empty`, `error` — a flow that ends in a list worth showing empty earns a second screen at `empty`, and a flow with a failure branch earns one at `error`.

## Why `out_dir` is exempt from `design lint`

`.devforgeai/explore/mockups/**` and `.explore-prototype/**` are the two sketch entries in the default `config.toml` `[frontend].exclude` list, and `design lint` skips both prefixes in every run. A wireframe drawn before a brand kit exists has no token to resolve against, so the literal colours in `templates/sketch-screen.html` are the intended content rather than a violation.

The directory is Explore's `out_dir` and is deleted by `devforgeai explore prune` on `kill` and on `promote`. On `promote` the files carry forward first: spec mode reads the matching mockups at step 13 as the source of a screen's `## Anatomy` rows.

## The brand sketch

`brand-sketch.json` holds `name`, `palette` (3 to 6 hex values, each `^#[0-9a-f]{6}$`), and `type_pair` (two family strings). It is a precursor to `brand/tokens.json`, not an instance of it: it carries no group structure, no light and dark pair, and no contrast guarantee. Brand mode step 5 reads it as the candidate palette, and the `From sketch` answer to the `Color` question seeds the 12 colour leaves from it, each adjusted until its row in `## Color` clears its ratio.

A request carrying `brand: null` writes no file and leaves `brand_sketch` as `null`.
