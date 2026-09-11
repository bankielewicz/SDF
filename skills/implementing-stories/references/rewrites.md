# The seven rewrites

Read this at step 7.8 and at step 10, before `refactor-surgeon` is invoked. The trigger, the target paths, and the two numbers arrive from the CLI: `build-lint`, `build-complexity`, or `build-antipatterns` with its `reason` string, `[build].complexity_max`, and `[build].duplication_max_percent`. This file says which of the seven `pattern` values answers which measured condition, and what each one leaves unchanged.

Every rewrite here keeps behaviour. The tests that were green before the rewrite are the ones green after it, and the criterion the cycle was implementing stays implemented. A rewrite that changes what a test reads is a different change and belongs in a cycle of its own.

## Choosing by trigger

| Trigger | What the CLI measured | Patterns that answer it |
|---|---|---|
| `build-complexity` | one body branches more ways than `[build].complexity_max` allows | `extract-function`, `replace-conditional`, `inline` |
| `build-lint` | a rule of `## Formatting` or `## Naming` failed on a named path and line | `rename`, `extract-function` |
| `build-antipatterns` | an `AP-nnn` detector matched a path and line | `move-to-layer`, `extract-type`, `remove-duplicate`, `replace-conditional` |

A duplication figure above `[build].duplication_max_percent` arrives under `build-complexity` with the repeated run named in `reason`, and `remove-duplicate` answers it.

## The seven

**`extract-function`** — a body that carries several steps becomes a body that names them. Take a contiguous run of the body that reads and writes a small set of local values, give it a name that says what the run produces, pass the values it reads, return the value it produces, and call it in place. The branch count of the original body drops by the branches the run held. This is the pattern for a body over the complexity ceiling whose branches are independent of each other.

**`extract-type`** — a set of values that travel together, and the rules that hold between them, become one named thing. The callers that passed the values one by one pass the thing instead. This answers a long argument list, and an `AP-nnn` whose detector names a group of fields repeated across files.

**`inline`** — a name that adds nothing is replaced by what it names. A function called once whose body is shorter than its signature, a variable assigned once and read once, an argument that is the same value at every call site. This lowers the count of things a reader holds and sometimes lowers the branch count, because a wrapper's own guard disappears with it.

**`rename`** — the name changes and nothing else does. This answers a `## Naming` rule the lint reported, and a name whose meaning drifted from what the code does. Every reference moves in the same rewrite; a rename that leaves one reference behind is a compile or resolution failure rather than a behaviour change, which the test run reports.

**`replace-conditional`** — a branch on a kind becomes a dispatch. When one body branches several ways on which kind of thing it holds, and the same branching appears in a second body, each kind takes its own implementation of the operation and the branch disappears from both bodies. This answers a high branch count that `extract-function` only redistributes, because the branching itself is the shape being removed.

**`move-to-layer`** — code sitting in the wrong layer moves to the right one. A decision found in an outward-edge file moves to the layer `## Layer dependency rules` assigns decisions; a storage detail found in a rules file moves to the edge. The destination is a `## Files` row the story already declares. This is the pattern for an `AP-nnn` whose detector names a layer violation.

**`remove-duplicate`** — repeated text becomes one definition and calls to it. The repeated run is extracted once at the layer both copies can name under `## Layer dependency rules`, and each copy becomes a call. Two runs that look alike and answer different questions are not this pattern: a single definition would then carry a flag, which raises the branch count rather than lowering it.

## When the agent declines

`refactor-surgeon` returns `status: declined` with one of three reasons rather than writing outside the declared set.

| `declined.reason` | What it means |
|---|---|
| `rewrite_leaves_declared_file_set` | the destination the pattern needs is a path the story's `## Files` table does not hold |
| `constraint_forbids_the_rewrite` | a `CON-nnn` in `## Constraints` rules out the shape the pattern produces |
| `threshold_breach_is_in_a_file_the_story_does_not_declare` | the path the CLI named is outside the story's file set |

A declined rewrite leaves the measurement where it was, and the `build-lint`, `build-complexity`, or `build-antipatterns` check carries it to the gate. The run continues to the next step either way.

## Recording the change

Each entry of `changes[]` carries the path, the pattern, and one line each for the shape before and after — the name that went away and the name that replaced it, or the run that was repeated and the definition that now holds it. Those two lines are what the report shows and what a later reader matches against the diff.
