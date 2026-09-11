# Recommendation targets

One recommendation names one target. The six kinds below are the whole set, and every
path is a path the installed project holds: `.claude/` carries the skills, the agents
and the hook block, `.devforgeai/` carries the two configuration files. The repository
the framework is built from is not on disk in a target project, so no recommendation
names a path under it.

| kind | path | key | What a change to it does |
|---|---|---|---|
| skill | `.claude/skills/<skill-name>/SKILL.md` | `""` | Changes the workflow, the judgment, or the wording one phase applies |
| subagent | `.claude/agents/<agent-name>.md` | `""` | Changes one agent's purpose, tools, model, input, or output schema |
| template | `.claude/skills/<skill-name>/templates/<file>` | `""` | Changes the document shape one phase writes |
| hook | `.claude/settings.json` | the hook event name | Changes when the CLI runs, or with which matcher |
| gate_threshold | `.devforgeai/gates.toml` or `.devforgeai/config.toml` | a dotted key | Changes a number a gate reads |
| framework_file | an installed path none of the five rows above names, e.g. `.claude/skills/<name>/references/<file>` | `""` | Changes a file none of the other five rows names |

The rows are ordered by specificity and a target takes the first row it matches. A recommendation whose `target.path` is `.devforgeai/gates.toml` or `.devforgeai/config.toml` takes `kind: gate_threshold` with the dotted key in `target.key`, whatever the change is; `framework_file` names neither of those two files.

## The floors

A `gate_threshold` recommendation carries `current_value` and `proposed_value` as strings. These keys have a compiled floor; a proposal below the floor is dropped before the report is written, and the drop is recorded in `open_questions`.

| File | Key | Floor |
|---|---|---|
| `config.toml` | `layer.domain.coverage_min` | 90.0 |
| `config.toml` | `layer.application.coverage_min` | 80.0 |
| `config.toml` | `layer.infrastructure.coverage_min` | 70.0 |
| `config.toml` | `layer.interface.coverage_min` | 60.0 |
| `config.toml` | `coverage.overall_min` | 75.0 |
| `gates.toml` | `verifier_pass.min_ratio` | 1.0 |

## The nine skill names in a path

`exploring-ideas`, `discovering-requirements`, `establishing-context`, `planning-work`, `implementing-stories`, `validating-quality`, `releasing-software`, `designing-interfaces`, `improving-framework`.

A skill's directory name is the one above; its slash command is its frontmatter `name`
(`explore`, `discover`, `constitute`, `plan`, `build`, `verify`, `release`, `design`,
`reflect`). A `skill` or `template` path uses the directory name.
