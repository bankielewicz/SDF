# Tests that cross a layer boundary

Read this at step 8, before `integration-test-writer` is invoked. The unit tests of step 7 each pin one criterion against one unit. This file says what a test at a boundary adds beyond them, and which of the four `boundary` values of the agent's `scenarios[]` a given test carries.

A boundary test exercises two layers of `## Layer dependency rules` through their edge. It composes units that step 7 already left green, so a failure in one names the edge rather than either side of it.

## The four boundary values

`integration-test-writer` labels every scenario with one of four values. The value is the edge the scenario crosses, read from the story's `## Layer` and its `## Files` rows.

| `boundary` | The edge | What a scenario at this edge reads |
|---|---|---|
| `interface-application` | an inbound request or a rendered interaction reaching one application entry point | the result the entry point returned, shaped for the inbound surface: a status, a body field, a rendered region |
| `application-infrastructure` | an application entry point reaching storage, a file, or a queue through the shape the lower layer declared | the state the edge left behind after the call, read back through the same edge |
| `application-domain` | an application entry point reaching the rules that decide an outcome | the decision the rules produced for one supplied state, including the rejection case |
| `interface-external` | an inbound surface reaching an outward call the project does not own | the project's own behaviour under each outcome the outward call can return |

A story crossing no edge returns `status: not-applicable` with `not_applicable_reason: story_crosses_no_layer_boundary`. A story crossing an edge with no `## Files` row of kind `test` at that edge returns `no_declared_test_path_for_the_boundary`. Both leave step 8 with no file written and the run continues at step 9.

## What one scenario holds

A scenario names the criteria it carries in `acs[]`, so a scenario is written for a set of `AC-nnn` lines rather than for a unit. Three parts:

1. **A starting state placed through the edge below.** The rows, files, or queue entries the criteria assume are written by the same path the code under test reads them by. A state placed by reaching past that path tests a shape no run produces.
2. **One call at the edge above.** One request, one entry-point call, one interaction. A scenario with two calls at the top is two scenarios, unless the second call is what a criterion's `When` clause names, in which case the pair is the action.
3. **Assertions on both sides of the edge.** The value the call returned, and the state the edge below holds afterward. A scenario that reads only the return value passes when the write silently did nothing.

## The three shapes worth a scenario

- **A pair that has to move together.** A criterion whose `Then` clause names two outcomes — a stored record and a notification, a decremented count and a created row — is one scenario asserting both. The unit tests of step 7 saw one each.
- **The rejection that has to leave nothing behind.** A criterion whose `Then` clause names a refusal gets a scenario that performs the refused action and then reads the state below, asserting it is the state from before the call.
- **The ordered path.** Criteria that name a sequence — a state reached, then acted on, then read — become one scenario running the sequence in order, asserting the outcome each criterion names at the point that criterion names it.

## What stays out

An outward call the project does not own is answered by a stand-in the test supplies, at the edge named in `## Approved dependencies` and no deeper. The stand-in returns each outcome the criteria name, one scenario per outcome, and the scenario asserts what the project did with it rather than what the stand-in received.

State placed by one scenario does not survive into the next. Each scenario writes what it reads and leaves the edge below at the state it found, so the order the scenarios run in changes no result.

Paths outside the story's `## Files` rows of kind `test` are not written here: a scenario that needs one is reported through the run rather than created, because the `PreToolUse` check refuses the write and the declared set is Plan's output.
