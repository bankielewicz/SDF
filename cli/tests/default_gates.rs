//! The default `gates.toml` `init` writes is the spec's `## Gate` block,
//! verbatim.
//!
//! `templates/gates.default.toml` is compiled into the binary with
//! `include_str!`. This test re-extracts the block from `specs/01-cli.md` and
//! compares the two byte for byte, so the requirement is self-checking rather
//! than asserted by eye.

use devforgeai::gates::{self, DEFAULT_GATES};

/// The single ```toml fence under the `## Gate` heading of the CLI spec.
fn spec_gate_block() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("specs")
        .join("01-cli.md");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));

    let mut in_gate_section = false;
    let mut in_fence = false;
    let mut out: Vec<&str> = Vec::new();

    for line in text.lines() {
        if line.starts_with("## ") {
            if in_fence {
                panic!("the ## Gate fence did not close before the next heading");
            }
            in_gate_section = line.trim() == "## Gate";
            continue;
        }
        if !in_gate_section {
            continue;
        }
        if line.trim_end() == "```toml" && !in_fence {
            in_fence = true;
            continue;
        }
        if line.trim_end() == "```" && in_fence {
            break;
        }
        if in_fence {
            out.push(line);
        }
    }

    assert!(!out.is_empty(), "the ## Gate block was found");
    let mut joined = out.join("\n");
    joined.push('\n');
    joined
}

#[test]
fn the_default_file_is_the_spec_block_byte_for_byte() {
    assert_eq!(
        DEFAULT_GATES,
        spec_gate_block(),
        "templates/gates.default.toml has drifted from specs/01-cli.md ## Gate"
    );
}

#[test]
fn the_default_file_holds_eight_gates_and_sixty_four_checks() {
    // The count the spec asserts immediately after the fence. It catches
    // transcription loss that a diff of two equal-but-wrong files would not.
    let g = gates::parse(DEFAULT_GATES).expect("the default file parses");
    assert_eq!(g.gate.len(), 8, "eight gates");
    let checks: usize = g.gate.iter().map(|x| x.check.len()).sum();
    assert_eq!(checks, 64, "sixty-four checks");
}

#[test]
fn the_default_file_names_one_gate_per_phase_and_none_for_design() {
    let g = gates::parse(DEFAULT_GATES).expect("parse");
    let phases: Vec<&str> = g.gate.iter().map(|x| x.phase.as_str()).collect();
    assert_eq!(
        phases,
        vec![
            "explore",
            "discover",
            "constitute",
            "plan",
            "build",
            "verify",
            "release",
            "reflect"
        ]
    );
    assert!(
        g.gate_for("design").is_err(),
        "Design is not a phase and holds no gate"
    );
}

// ------------------------------------------ required keys and enum-valued keys

/// One gate carrying one check, for the load-time rules below.
fn one_check(body: &str) -> String {
    format!(
        "schema = \"devforgeai/gates/1\"\ncli_min_version = \"1.0.0\"\n\n[[gate]]\nphase = \"explore\"\nrequires = \"\"\non_fail = \"fail\"\nsend_back_to = \"\"\n\n  [[gate.check]]\n{body}"
    )
}

/// The error `gates::parse` raises for `body`, or a panic naming the pass.
fn parse_err(body: &str) -> devforgeai::errors::CliError {
    gates::parse(&one_check(body)).expect_err("the check is refused at load")
}

#[test]
fn a_check_missing_a_required_key_is_e302() {
    // One case per required key of the spec's kind table: a check whose
    // required key is absent satisfied the compiled minimum and then passed
    // vacuously, which is the shared root of eleven findings.
    let cases: &[(&str, &str, &str)] = &[
        ("file_exists", "paths", "  kind = \"file_exists\"\n  id = \"x\"\n  min_count = 1\n"),
        ("field_in_enum", "values", "  kind = \"field_in_enum\"\n  id = \"x\"\n  path = \"a.yaml\"\n  field = \"status\"\n"),
        ("field_in_enum", "field", "  kind = \"field_in_enum\"\n  id = \"x\"\n  path = \"a.yaml\"\n  values = [\"a\"]\n"),
        ("field_is_date", "field", "  kind = \"field_is_date\"\n  id = \"x\"\n  path = \"a.yaml\"\n"),
        ("length_between", "field", "  kind = \"length_between\"\n  id = \"x\"\n  path = \"a.yaml\"\n  min = 1\n"),
        ("set_cover", "universe", "  kind = \"set_cover\"\n  id = \"x\"\n  cover = \"epics[].requirements\"\n"),
        ("row_count_between", "section", "  kind = \"row_count_between\"\n  id = \"x\"\n  path = \"a.md\"\n  min = 1\n"),
        ("column_matches", "pattern", "  kind = \"column_matches\"\n  id = \"x\"\n  path = \"a.md\"\n  section = \"S\"\n  column = \"ID\"\n"),
        ("column_contains_all", "state_field", "  kind = \"column_contains_all\"\n  id = \"x\"\n  path = \"a.md\"\n  section = \"S\"\n  column = \"ID\"\n"),
        ("elapsed_days_at_most", "limit_field", "  kind = \"elapsed_days_at_most\"\n  id = \"x\"\n  started_field = \"explore.started_at\"\n"),
        ("verifier_pass", "verifiers", "  kind = \"verifier_pass\"\n  id = \"x\"\n  min_ratio = 1.0\n"),
        ("report_metric", "metric", "  kind = \"report_metric\"\n  id = \"x\"\n  op = \"eq\"\n  value = 0\n"),
        ("no_cycle", "docs", "  kind = \"no_cycle\"\n  id = \"x\"\n  from = \"id\"\n  to = \"deferrals[].target\"\n"),
        ("no_cycle", "to", "  kind = \"no_cycle\"\n  id = \"x\"\n  docs = [\"a.yaml\"]\n  from = \"id\"\n"),
        ("yaml_cites", "into", "  kind = \"yaml_cites\"\n  id = \"x\"\n  doc = \"a.yaml\"\n  from = \"recommendations\"\n  field = \"observations\"\n"),
        ("no_threshold_decrease", "doc", "  kind = \"no_threshold_decrease\"\n  id = \"x\"\n  from = \"recommendations\"\n"),
    ];
    for (kind, key, body) in cases {
        let err = parse_err(body);
        assert_eq!(err.code(), "DFA-E302", "{kind} without {key}");
        let message = err.diag().expect("diag").message.clone();
        assert!(
            message.contains(key),
            "{kind} without {key}: the message names the absent key; it was {message}"
        );
    }
}

#[test]
fn a_key_outside_its_enum_is_e302() {
    let cases: &[(&str, &str)] = &[
        (
            "scope",
            "  kind = \"story_valid\"\n  id = \"x\"\n  scope = \"evrything\"\n",
        ),
        (
            "scope",
            "  kind = \"antipattern_clean\"\n  id = \"x\"\n  scope = \"Story\"\n",
        ),
        (
            "min_severity",
            "  kind = \"antipattern_clean\"\n  id = \"x\"\n  min_severity = \"critical\"\n",
        ),
        (
            "source",
            "  kind = \"coverage_min\"\n  id = \"x\"\n  source = \"raed\"\n",
        ),
        (
            "op",
            "  kind = \"report_metric\"\n  id = \"x\"\n  metric = \"a.b\"\n  op = \"ge\"\n",
        ),
    ];
    for (key, body) in cases {
        let err = parse_err(body);
        assert_eq!(err.code(), "DFA-E302", "{key} outside its enum");
        assert!(
            err.diag().expect("diag").message.contains(key),
            "the message names the key"
        );
    }
}

#[test]
fn ids_resolve_refuses_a_half_configured_or_doubled_form() {
    // `from` with no `to` silently dropped `from` and ran the whole-index
    // prefix scan instead, with no diagnostic.
    let err =
        parse_err("  kind = \"ids_resolve\"\n  id = \"x\"\n  from = \"requirements[].actor\"\n");
    assert_eq!(err.code(), "DFA-E302");
    let err = parse_err(
        "  kind = \"ids_resolve\"\n  id = \"x\"\n  from = \"a\"\n  to = \"b\"\n  prefixes = [\"REQ\"]\n",
    );
    assert_eq!(err.code(), "DFA-E302");
}

#[test]
fn file_exists_refuses_a_neutralised_min_count() {
    let err = parse_err(
        "  kind = \"file_exists\"\n  id = \"x\"\n  paths = [\"a.md\"]\n  min_count = 0\n",
    );
    assert_eq!(err.code(), "DFA-E302");
    let err = parse_err(
        "  kind = \"file_exists\"\n  id = \"x\"\n  paths = [\"a.md\"]\n  absent = true\n  min_count = 1\n",
    );
    assert_eq!(err.code(), "DFA-E302");
}

#[test]
fn fields_present_refuses_fields_without_a_collection() {
    // With `path` and no `collection`, `fields` is read and never used.
    let err = parse_err(
        "  kind = \"fields_present\"\n  id = \"x\"\n  path = \"a.yaml\"\n  fields = [\"name\", \"date\"]\n",
    );
    assert_eq!(err.code(), "DFA-E302");
}

#[test]
fn a_null_when_on_a_kind_with_no_field_is_e302() {
    // `null_when` and `empty_when` are rules about the check's own `field`, so
    // a kind that defines none can carry neither: the rule would read the
    // document root, which no document can satisfy.
    let err = parse_err(
        "  kind = \"row_count_between\"\n  id = \"x\"\n  path = \"a.md\"\n  section = \"S\"\n  empty_when = { path = \"b.yaml\", field = \"decision\", equals = \"kill\" }\n",
    );
    assert_eq!(err.code(), "DFA-E302");
}

#[test]
fn a_condition_naming_two_subjects_is_e302() {
    let err = parse_err(
        "  kind = \"file_exists\"\n  id = \"x\"\n  paths = [\"a.md\"]\n  skip_when = { doc_exists = \"b.yaml\", state_field = \"explore.remedy_flows\" }\n",
    );
    assert_eq!(err.code(), "DFA-E302");
    // `equals` with no subject to read it from is always false, so the check it
    // guards is always skipped.
    let err = parse_err(
        "  kind = \"file_exists\"\n  id = \"x\"\n  paths = [\"a.md\"]\n  required_when = { equals = \"kill\" }\n",
    );
    assert_eq!(err.code(), "DFA-E302");
    // `doc_exists` tests existence alone, so `equals` beside it is dropped.
    let err = parse_err(
        "  kind = \"file_exists\"\n  id = \"x\"\n  paths = [\"a.md\"]\n  skip_when = { doc_exists = \"b.yaml\", equals = \"kill\" }\n",
    );
    assert_eq!(err.code(), "DFA-E302");
    // A `path` condition reads a field inside the document it names.
    let err = parse_err(
        "  kind = \"file_exists\"\n  id = \"x\"\n  paths = [\"a.md\"]\n  skip_when = { path = \"b.yaml\", equals = \"kill\" }\n",
    );
    assert_eq!(err.code(), "DFA-E302");
}

#[test]
fn a_gate_phase_outside_the_enum_is_refused() {
    // An unrecognised phase fell through every compiled table to an empty
    // slice, so it was required to carry no check kind and bound by no floor.
    let text = "schema = \"devforgeai/gates/1\"\ncli_min_version = \"1.0.0\"\n\n[[gate]]\nphase = \"invent\"\nrequires = \"\"\non_fail = \"fail\"\nsend_back_to = \"\"\n";
    let err = gates::parse(text).expect_err("the phase is outside the enum");
    assert_eq!(err.code(), "DFA-E302");
}

#[test]
fn the_default_file_satisfies_every_load_time_rule() {
    // The shipped file is the reference case for all of the above.
    gates::parse(DEFAULT_GATES)
        .expect("the default file parses")
        .validate_minimums()
        .expect("the default file satisfies the compiled minimums");
}

#[test]
fn every_default_metric_into_an_agent_field_reads_through_payload() {
    // A registered verifier prints one object: the envelope keys at the top of
    // the ingested block and the agent's own fields under `payload`. Every
    // `report_metric` the default gates carry names an agent field, so each
    // must read through that segment; `verifier_pass` is unaffected, since
    // `passed`, `total` and `unit` stay at the top of the block.
    let g = gates::parse(DEFAULT_GATES).expect("parse");
    let mut seen = 0usize;
    for gate in &g.gate {
        for c in &gate.check {
            if c.kind != "report_metric" {
                continue;
            }
            let metric = c.str_key("metric", "");
            seen += 1;
            assert!(
                metric.starts_with("verifiers."),
                "gate '{}' check '{}' reads {metric}, which is not a verifier field",
                gate.phase,
                c.id
            );
            assert!(
                metric.contains(".payload."),
                "gate '{}' check '{}' reads {metric}, which skips the payload segment",
                gate.phase,
                c.id
            );
        }
    }
    assert_eq!(seen, 3, "the three constitute metrics");
}

#[test]
fn every_verifier_the_default_gates_name_is_registered() {
    // A gate naming a subagent absent from the registry fails with DFA-E316
    // rather than passing silently, so the two lists agree by construction.
    let g = gates::parse(DEFAULT_GATES).expect("parse");
    let registry: std::collections::BTreeSet<String> = devforgeai::config::default_verifiers()
        .into_iter()
        .map(|v| v.name)
        .collect();

    for gate in &g.gate {
        for c in &gate.check {
            if c.kind != "verifier_pass" {
                continue;
            }
            for name in c.arr_key("verifiers", &[]) {
                assert!(
                    registry.contains(&name),
                    "gate '{}' check '{}' names '{name}', which config.toml [[verifier]] does not register",
                    gate.phase,
                    c.id
                );
            }
        }
    }
}

#[test]
fn the_discover_gate_can_send_a_brief_back_to_explore() {
    let g = gates::parse(DEFAULT_GATES).expect("parse");
    let discover = g
        .gate
        .iter()
        .find(|x| x.phase == "discover")
        .expect("the discover gate");

    // The skill's `## Send-back` says an actorless flow or a contradiction
    // sends the brief back. Without a check whose `on_fail` is `send_back`,
    // `gate check --phase discover` can only ever fail, and the send-back the
    // skill documents is unreachable.
    assert_eq!(discover.send_back_to, "explore");
    let check = discover
        .check
        .iter()
        .find(|c| c.kind == "verifier_pass")
        .expect("a verifier_pass check on the discover gate");
    assert_eq!(check.id, "flow-integrity");
    assert_eq!(check.on_fail, "send_back");
    assert_eq!(
        check
            .keys
            .get("verifiers")
            .and_then(|v| v.as_str().map(str::to_string)),
        None,
        "verifiers is a list, not a string"
    );
}
