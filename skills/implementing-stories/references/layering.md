# Placing code in a layer

Read this at step 7.4, before the first source path of a criterion is written. The story fixes one `## Layer` value and one `## Files` set; this file says which part of the code for a criterion belongs at which declared path, and what a layer rule decides that a path pattern does not.

The layer names, their order, and the edges between them come from `.devforgeai/context/source-tree.md` `## Layers` and `.devforgeai/context/architecture-constraints.md` `## Layer dependency rules`. The four names used below — `domain`, `application`, `infrastructure`, `interface` — are the names the shipped context templates carry. A project whose `## Layers` section names others reads this file with its own names substituted, because the rule below is about direction and not about vocabulary.

## What each layer holds

| Layer | Holds | Does not hold |
|---|---|---|
| `domain` | the state of a thing the project reasons about, the rules that decide whether a change to that state is allowed, and the values those rules compare | any reference to storage, transport, a clock the caller does not pass, or a rendering surface |
| `application` | one entry point per use case: gather the inputs, call the domain, record the outcome, return a result | the rules themselves, and any shape that belongs to one transport |
| `infrastructure` | the implementation of an outward edge — storage, an outbound call, a file, a queue — behind the shape the layer above named | any branch on state that the domain also branches on |
| `interface` | the inbound edge: parse a request or render a screen, call one application entry point, present its result | any decision the application layer is the one entry point for |

A criterion whose `Then` clause names a stored row, a returned value, or a status touches the first three; a criterion whose `Then` clause names a rendered region touches the fourth. The `## Files` table already says which paths exist for the story, so the placement question is which of the declared paths a given piece of the change belongs in, not whether a new path is warranted.

## The one direction rule

The edges in `## Layer dependency rules` run one way. A layer names the layer below it and does not name the layer above it. `domain` sits at the end of every chain and names none of the others.

Two shapes satisfy that rule when code at the bottom has to reach an outward edge:

1. The layer that owns the rule declares the operation it needs — a name, its inputs, its result — and the layer that owns the edge supplies a body for that declaration. The declaration sits in the lower layer; the body sits in the higher one.
2. The caller passes the value in. A time, a generated identifier, or a random draw arrives as an argument rather than being read where the rule runs, which also makes the rule readable by a test that passes a fixed value.

A construction expression that names a storage type, a transport type, or an outbound client inside `domain` or `application` reverses an edge even when every declared path is respected, because the name of the lower layer's file then resolves to a symbol the upper layer defines.

## Where a criterion's parts land

Take one `AC-nnn` and split it at its three clauses:

- The `Given` clause names a state. The state lives in `domain`; the path that puts it there for a test lives in `infrastructure` or in the test's own setup.
- The `When` clause names an action. One `application` entry point carries it, and the `interface` path that a request or a click reaches calls that entry point and adds nothing else.
- The `Then` clause names an outcome a test reads. A rule that decides the outcome sits in `domain`; the record of the outcome sits wherever the `## Files` row of kind `source` for the story's layer points.

Two criteria of one story that name the same rule get one implementation of that rule, called twice. A second copy of a rule inside a second entry point is the duplication `devforgeai gate check` measures at `[build].duplication_max_percent` and the shape `refactor-surgeon` removes at step 7.8.

## What blocks placement

`backend-implementer` returns `status: blocked` rather than writing outside the declared set. The four reasons map onto this file as follows.

| `blocked.reason` | What it means here |
|---|---|
| `no_declared_path_fits` | the part of the change described above has no `## Files` row at the layer it belongs to |
| `constraint_forbids_the_only_shape` | a `CON-nnn` in `## Constraints` rules out both shapes of the direction rule for this edge |
| `dependency_not_approved` | the edge needs a name absent from `## Approved dependencies`, or present in `## Forbidden dependencies` |
| `test_asserts_outside_declared_files` | the failing test reads an outcome from a path the story does not declare |

Each of the four ends the cycle for that criterion and travels to the report as a finding with the `CON-nnn` or `AP-nnn` ids that produced it. The story's `## Files` table is Plan's output, so a missing row is a Plan defect and reaches Plan through the report rather than through an edit here.
