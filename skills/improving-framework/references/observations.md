# Observations

Read this before workflow step 6, when the returns of steps 3 and 4 are merged and each one takes an id. It carries the five kinds, which part of the aggregate each is detected from, what `count` and `metric` mean per kind, how severity is set, and the field-by-field shape of an `observations[]` entry.

Every figure in an observation is copied from `devforgeai report aggregate --json`. The counting, the timestamp differencing, and the check-id tallying happen in the binary, which is why the two mining agents receive arrays that are already summed and spend their turn on which of those sums names a pattern.

## The five kinds

| `kind` | Detected from | `count` means | `metric` |
|---|---|---|---|
| `friction` | `sessions.commands[]`: a command re-typed inside one session with the same `$1` and no intervening gate `PASS` | re-typings | `""` |
| `repeated_send_back` | `send_backs[]`: two or more entries sharing `from`, `to`, and `id` | send-backs on that triple | `""` |
| `gate_failure` | `gate_failures[]`: one entry per failing `check_id` | failures of that check | `""` |
| `phase_time` | `phase_time[]`: one entry per phase in the window | runs of that phase | the total, as `<n>m <n>s` |
| `verifier_unparsed` | `verifier_failures[]`: a `status` of `unparsed` (`DFA-E410`) or an unregistered name (`DFA-W411`) | reports carrying that status | `""` |

`observation-miner` returns the three report-derived kinds: `gate_failure`, `phase_time`, `verifier_unparsed`. `session-pattern-reader` returns the two session-derived kinds: `repeated_send_back` and `friction`. A kind has one producer, so the merge at step 6 concatenates two lists and resolves nothing.

`metric` is `""` for four of the five kinds. Only `phase_time` fills it, and the string fits inside 20 characters.

A kind with no repeat notion — `gate_failure` counted once, `verifier_unparsed` on a single report — carries `count: 1`. `repeated_send_back` and `friction` start at 2, because one occurrence of either is an incident rather than a pattern.

## Severity

`low`, `medium`, `high`, a closed three. The value ranks what the observation cost the run, so the ordering at step 6 puts the expensive lines first:

- `high` — the pattern repeated three or more times, spanned two sessions, or failed a blocking check in more than half the runs of its phase.
- `medium` — the pattern repeated twice inside one session, or the check failed more than once.
- `low` — one failure, a re-typed argument with no failed gate between the typings, or a phase whose time sits below the window's median.

## The entry

| Field | Type | Constraint |
|---|---|---|
| `id` | string | `^OBS-[0-9]{3}$`, from `devforgeai doc validate --allocate OBS` at step 6 |
| `kind` | string | one of the five above |
| `severity` | string | `low`, `medium`, `high` |
| `phase` | string | a phase name, or `""` when the observation spans phases |
| `summary` | string | one line, 1 to 120 characters, naming the id or the check the pattern turned on |
| `detail` | string | 1 to 400 characters, saying what the evidence shows |
| `count` | integer | 1 or more, by the table above |
| `metric` | string | `""` unless `kind` is `phase_time` |
| `sources` | list of string | 1 or more; report paths, session ids, or both |
| `evidence` | list of object | at most 5 entries in time order, each with `path`, `line`, `at` |

`sources` is what the `reflect-obs-cites-source` check reads. Each entry resolves either to a `sources.reports[].path` value or to a `sources.sessions.files[].session_id` value of the same document, and an entry outside that union is `DFA-E347`. A report-derived observation cites report paths; a session-derived one cites session ids; an observation drawn from both cites both.

`evidence[].line` is 1-based in a session file and `0` for a report path, because a report is cited whole.

## Ordering and ids

Step 6 orders the merged list by `severity` descending, then by `kind` in the table order above, then runs `devforgeai doc validate --allocate OBS` once per observation, in that order. The ids are global and monotonic across every reflect report, so a `REC-nnn` in a later report can cite an `OBS-nnn` an earlier one defined and one id points at one observation across the whole project. Exit 1 with `DFA-E215` means the prefix is exhausted and the run stops with the stderr line in hand.

An observation whose `sources` came back empty is dropped at step 6 and one line naming its `ref` goes into `open_questions`. The check that would otherwise catch it is `reflect-obs-cites-source`, and dropping the entry keeps the gate reading a document rather than a defect.

## The shapes to copy

`templates/reflect-report.yaml` carries `observations: []` and `technical_debt.groups: []`,
because an empty window writes those keys empty. These are the shapes a filled entry takes.

One `observations[]` entry:

```yaml
observations:
  - id: OBS-000
    kind: <friction|repeated_send_back|gate_failure|phase_time|verifier_unparsed>
    severity: <low|medium|high>
    phase: <phase|>
    summary: <1 to 120 chars>
    detail: <1 to 400 chars>
    count: 1
    metric: ""
    sources: []
    evidence:
      - path: <path>
        line: 0
        at: <RFC 3339 UTC>
```

One `technical_debt.groups[]` entry, which `debt-aggregator` returns at step 5. A group is
one `CON-nnn` or `AP-nnn`, or the `none` group for a deferral record citing neither:

```yaml
technical_debt:
  groups:
    - constraint: <CON-nnn|AP-nnn|none>
      kind: <constraint|anti_pattern|none>
      count: 1
      oldest_days: 0
      items:
        - story: STORY-000
          dod_item: <1 to 120 chars>
          deferred_at: <YYYY-MM-DD>
          age_days: 0
          reason: <1 to 200 chars>
          report: <path>
```

`deferred_at`, `age_days`, and `report` are written by `report aggregate` from the QA
report's `deferrals[].opened_on`; this run copies them and computes none of them.
