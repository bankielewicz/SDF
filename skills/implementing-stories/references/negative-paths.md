# Turning a failure criterion into an assertion

Read this at step 7.1 for a criterion whose `Then` clause names a refusal, an error, or an empty result. A criterion that names success has one obvious outcome to read; a criterion that names failure has three, and reading the wrong one produces a test that passes whatever the code does.

## The three readings of a failure

A failure criterion fixes up to three things, and the test reads each one the criterion names.

1. **The signal.** How the caller learns the action did not succeed: a raised condition, a returned marker, a status, an empty collection. The criterion's `Then` clause names which, and the test reads that one. A test that accepts any signal passes when the code changes from one to another, which is a change a caller feels.
2. **The identity.** Which failure it is. A criterion that names a reason — a value out of range, a missing reference, a state that forbids the action — has a test that distinguishes that failure from the other failures the same action can produce. Reading only "it failed" lets a different failure satisfy the test.
3. **What is left behind.** The state after the refusal. A criterion whose `Given` clause names existing state has a test that reads that state after the action and asserts it is unchanged. A refusal that half-completes passes every assertion about the signal.

The third reading is the one a success-shaped test leaves uncovered, and it is where a criterion of the form "the order is rejected and the stock is unchanged" lives.

## Four kinds of path behind one action

One `AC-nnn` names one outcome. Across the criteria of a story, the outcomes fall into four kinds, and the kinds are worth naming because a story whose criteria all fall into the first is a story whose failure behaviour no test reads.

| Kind | The criterion names |
|---|---|
| success | the action completing with the value the caller wanted |
| refused | the action declining, with the state from before it left intact |
| interrupted | the action meeting a condition it does not handle, and what the caller sees when it does |
| edge | the action at the limit of a range the criterion fixes: none, one, the largest allowed, the first disallowed |

Step 7 writes one test per criterion, so the spread across the four kinds is decided by the criteria Plan wrote. A story whose criteria cover only the first kind is reported as a `warn` finding by `story-invest-auditor` at plan time rather than repaired by writing tests for criteria that do not exist.

## Edges worth a criterion

An edge criterion fixes a boundary and the test reads both sides of it. Three shapes cover the boundaries that appear in acceptance criteria:

- **A count with a limit.** The criterion fixes the limit; the test reads the last allowed count and the first disallowed count. The value one short of the limit adds nothing the last allowed value does not already establish.
- **A collection.** The criterion fixes what happens with no entries, and what happens with one. Empty is the reading most often absent from a success-shaped test, because the code path for empty is frequently a different path.
- **An absent value.** The criterion fixes what the action does when an input the caller may omit is omitted. The test supplies the omission rather than a placeholder that stands for it.

## Cleanup that runs either way

An action that acquires something — a handle, a lock, a reserved count — and then fails releases it. A criterion that names the release is read by a test that triggers the failure and then reads the thing, asserting it is available. The test triggers the failure through the input the criterion names rather than by reaching into the code's own path, so the assertion holds for any implementation of the same criterion.

## What stays out of a failure test

The message text a failure carries is asserted when the criterion fixes it and left alone when the criterion does not, because a message the criterion did not name is wording rather than behaviour, and a test that pins it fails on every rewording.

A failure the criterion does not name gets no test in this cycle. It reaches the run as an `open_questions` line or as a Verify finding about a criterion that says less than its `REQ-nnn`, which is the `spec-gap` category, and it travels to Plan through the report.
