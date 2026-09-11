# The shape of a test for one criterion

Read this at step 7.1, before `ac-test-writer` is invoked for a criterion. One `AC-nnn` produces one failing test case. This file says how the criterion's three clauses become the three parts of that test case, and what makes a `Then` clause readable enough to assert.

The test case is written into a path the story's `## Files` table declares with `Kind` `test`, and the cases of several criteria may land in one declared file. The rules the file follows — naming, layout, where a case goes — come from `coding-standards.md` `## Testing standards` and `source-tree.md` `## Naming conventions`, which the prompt carries. Nothing here names a runner, an assertion library, or a file extension.

## Three clauses, three parts

A criterion reads `Given <state> When <action> Then <outcome>`. The test has the same three parts in the same order, separated so a reader can point at each.

**Arrange, from `Given`.** Build the state the clause names and nothing beyond it. A value the clause does not name is either absent or set to the simplest value that lets the action run, and that choice is visible in the test rather than hidden behind a helper. State the criterion shares with a sibling criterion is built by each test case for itself, so one case's outcome does not depend on another's having run.

**Act, from `When`.** One call. The clause names one action, so the test performs one and holds its result. A second call before the assertion is either part of the arrangement, in which case it belongs above, or a second criterion, in which case it belongs in its own test.

**Assert, from `Then`.** Read the outcome the clause names and compare it to the value the clause fixes. One criterion's assertions all read one outcome: a returned value, a stored row, a status, a count, an emitted record, or a rendered region. A test that asserts the outcome and then asserts three other things it noticed is carrying claims no criterion made, and those claims fail later for reasons no criterion explains.

## Naming the case

The case name carries the criterion id and the outcome, so a failure line names the criterion without the reader opening the file. The order of the two and the separator between them come from `## Naming conventions`; the content is the id and a short phrase from the `Then` clause.

## What a stand-in replaces

The action reaches outward edges the project owns and edges it does not. An edge the project does not own is replaced by a stand-in that returns the value this criterion assumes; an edge the project owns is exercised as written, because a stand-in in its place tests the stand-in.

Two things are worth reading off the stand-in when the criterion names them: what the code under test passed to it, and how many times the code called it. A criterion whose `Then` clause names neither leaves both unasserted.

## An outcome a test can read

`ac-test-writer` returns `payload.testable` of `false` and a finding rather than writing a test it cannot assert. The five reasons and what each one looks like:

| `reason` | The criterion |
|---|---|
| `then_names_no_readable_outcome` | names a feeling, an appearance, or a state with no value, row, status, count, record, or region behind it |
| `when_names_no_action` | names a condition rather than something performed |
| `given_names_no_reachable_state` | names a state the declared file set has no path to produce |
| `contradicts_other_ac` | fixes a value a sibling criterion of the same story fixes differently |
| `outcome_outside_declared_files` | names an outcome produced by a path the story's `## Files` table does not hold |

Each of the five is a `block` finding cited by the `AC-nnn`, and it routes to Plan rather than being repaired here, because a criterion is Plan's output. A `contradicts_other_ac` reason emits one finding for the criterion under test and one for each id in `payload.conflicts_with`.

## The test fails first

The test is written before the code that satisfies it, and it is run before that code is written. A test that passes on its first run is reading something other than the outcome the criterion names — a default value, a stale record, an assertion that holds for every input — and the run treats the first-run result as part of the criterion's evidence rather than as a formality.
