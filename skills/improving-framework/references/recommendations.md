# Recommendations

Read this before workflow step 7, and again at step 9 when the `dropped[]` entries become `open_questions` lines. It carries the six target kinds, the compiled floors a `gate_threshold` proposal is measured against, the shape of a `recommendations[]` entry, and what this skill does with a recommendation once it is written.

`templates/rec-targets.md` is the table `recommendation-drafter` receives in its prompt. This document says how the run uses what comes back.

## A recommendation is prose

Reflect applies no change. A `REC-nnn` names one file and one change to it; the change happens when the user runs the next command against that file, and this run leaves every one of those files byte-for-byte as it found them.

The files a recommendation can name are the framework files the target project holds on disk: `.claude/skills/<name>/SKILL.md` and its `templates/`, `.claude/agents/<name>.md`, the `hooks` block of `.claude/settings.json`, and the two configuration files `.devforgeai/gates.toml` and `.devforgeai/config.toml`, which a `gate_threshold` recommendation names because a threshold lives there. A recommendation names no path outside those, because `/reflect` runs in the target project and the repository the framework was built from is not part of it. A target project with a `skills/` or `specs/` directory of its own holds its own code there, not this framework's. No recommendation targets a phase's document — a requirement, a context file, an ADR, a story, a UI spec — because a defect in one of those is a send-back, and this skill emits none.

The one file this run writes is `.devforgeai/reports/reflect-<date>.yaml`. The CLI writes a second, `.devforgeai/reports/<date>-reflect.yaml`, as the gate report for the run.

## The six target kinds

| `kind` | `path` shape | `key` |
|---|---|---|
| `skill` | `.claude/skills/<skill-name>/SKILL.md` | `""` |
| `subagent` | `.claude/agents/<agent-name>.md` | `""` |
| `template` | `.claude/skills/<skill-name>/templates/<file>` | `""` |
| `hook` | `.claude/settings.json` | the hook event name, e.g. `Stop` |
| `gate_threshold` | `.devforgeai/gates.toml` or `.devforgeai/config.toml` | a dotted key, e.g. `layer.domain.coverage_min` |
| `framework_file` | an installed path none of the five rows above names, e.g. `.claude/skills/<name>/references/<file>` | `""` |

There is no `command` kind. Each skill is its own entry point: its frontmatter `name` is the slash command, and a change to the preamble, the argument hint, or the tool grant is a change to `SKILL.md`, which the `skill` row already names.

The nine skill directory names that fill a `.claude/skills/<skill-name>/` path: `exploring-ideas`, `discovering-requirements`, `establishing-context`, `planning-work`, `implementing-stories`, `validating-quality`, `releasing-software`, `designing-interfaces`, `improving-framework`.

The rows are ordered by specificity and a target takes the first row it matches. A recommendation whose `target.path` is `.devforgeai/gates.toml` or `.devforgeai/config.toml` takes `kind: gate_threshold` with the dotted key in `target.key`, whatever the change is; `framework_file` names neither of those two files.

`framework_file` is the row for a target that is none of the other five — a skill's `references/` file, an installed `agents.md`. Each is a real place a change lands, and none of them is a skill body, an agent, a template, a hook, or a threshold.

## The floors

A `gate_threshold` recommendation carries `current_value` and `proposed_value` as strings. These six keys have a floor compiled into the binary:

| File | Key | Floor |
|---|---|---|
| `config.toml` | `layer.domain.coverage_min` | 90.0 |
| `config.toml` | `layer.application.coverage_min` | 80.0 |
| `config.toml` | `layer.infrastructure.coverage_min` | 70.0 |
| `config.toml` | `layer.interface.coverage_min` | 60.0 |
| `config.toml` | `coverage.overall_min` | 75.0 |
| `gates.toml` | `verifier_pass.min_ratio` | 1.0 |

The same numbers reach the run as the aggregate's `floors` block, so the values measured against are the ones the binary holds rather than a copy in prose.

A proposal below its floor is dropped by `recommendation-drafter` before the document is written, and the drop lands in that agent's `dropped[]`. Step 9 writes each drop as one `open_questions` line in the form:

```
REC dropped: <key> proposed <value>, floor <value>
```

A proposal at or above its floor stays, so raising a threshold is a recommendation this skill can make. The `reflect-no-lowered-floor` check reads the written document and fails with `DFA-E348` on a lowered floor that survived, which is conventions §8 applied one step earlier — to the document that proposes the change rather than to the file that would carry it.

## The entry

| Field | Type | Constraint |
|---|---|---|
| `id` | string | `^REC-[0-9]{3}$`, from `devforgeai doc validate --allocate REC` at step 8 |
| `observations` | list of string | 1 or more `OBS-nnn`, each defined in the `observations` list of the same document |
| `target.kind` | string | one of the six above |
| `target.path` | string | project-relative, under `.claude/` or `.devforgeai/` |
| `target.key` | string | `""` for every kind but `gate_threshold` and `hook` |
| `change` | string | 1 to 300 characters, imperative, one change |
| `current_value` | string | `""` unless `kind` is `gate_threshold` |
| `proposed_value` | string | `""` unless `kind` is `gate_threshold` |
| `effort` | string | `small`, `medium`, `large` |
| `applies_to` | list of string | phase names; `[]` when the change is framework-wide |

`observations` is what the `reflect-rec-cites-obs` check reads. An id outside the document's own `observations[].id` set is `DFA-E347`; an empty list is `DFA-E346`. The ids are allocated globally and monotonically, so an id defined in an earlier report resolves for a reader and not for this check, which resolves inside one document.

One recommendation names one change. A group of observations that one edit answers is one entry citing them all; two edits to the same file are two entries.

## The shape to copy

`templates/reflect-report.yaml` carries `recommendations: []`, because a window that
produced no observation writes that key empty. This is the shape a filled entry takes:

```yaml
recommendations:
  - id: REC-000
    observations: [OBS-000]
    target:
      kind: <skill|subagent|template|hook|gate_threshold|framework_file>
      path: <path>
      key: ""
    change: <1 to 300 chars, imperative, one change>
    current_value: ""
    proposed_value: ""
    effort: <small|medium|large>
    applies_to: []
```
