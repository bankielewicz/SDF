//! The release-facing check kinds: `deploy_manifest`, `docs_cover`, and the
//! `release_stories` extras.

mod common;

use assert_cmd::Command;
use common::Project;

fn cli(p: &Project) -> Command {
    let mut c = Command::cargo_bin("devforgeai").expect("the binary builds");
    c.arg("--project").arg(p.root());
    c.current_dir(p.root());
    c
}

fn gate_json(p: &Project) -> serde_json::Value {
    let out = cli(p)
        .args([
            "--json", "gate", "check", "--phase", "release", "--id", "v0.3.0",
        ])
        .output()
        .expect("run");
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    serde_json::from_str(text.trim()).unwrap_or_else(|e| panic!("{e} in {text}"))
}

fn status_of(v: &serde_json::Value, id: &str) -> String {
    v["data"]["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .find(|c| c["id"] == id)
        .unwrap_or_else(|| panic!("check {id} in {v}"))["status"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

fn reason_of(v: &serde_json::Value, id: &str) -> String {
    v["data"]["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .find(|c| c["id"] == id)
        .unwrap_or_else(|| panic!("check {id} in {v}"))["reason"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

/// A release gate carrying the two required kinds plus `extra`.
fn gates(extra: &str) -> String {
    format!(
        r#"schema = "devforgeai/gates/1"
cli_min_version = "1.0.0"

[[gate]]
phase = "release"
requires = ""
on_fail = "fail"

  [[gate.check]]
  kind = "doc_valid"
  id = "release-docs"
  docs = ["releases/{{id}}.yaml"]

  [[gate.check]]
  kind = "file_exists"
  id = "release-file"
  paths = ["releases/{{id}}.yaml"]
  min_count = 1

{extra}"#
    )
}

/// A release document with the keys the two kinds read.
fn release(platform: &str, manifests: &[&str], rollback: &str, api: &[&str]) -> String {
    let entries: String = manifests
        .iter()
        .map(|m| format!("    - path: {m}\n      kind: manifest\n"))
        .collect();
    let pages: String = api.iter().map(|a| format!("    - {a}\n")).collect();
    format!(
        "schema: devforgeai/release/1\nid: v0.3.0\nphase: release\nstatus: released\nproduced_by: releasing-software\nconsumes: []\nopen_questions: []\nplatform:\n  target: {platform}\ndeploy:\n  root: deploy\n  manifests:\n{}  ci_workflow: ''\n  ci_check_name: ''\n  rollback: {}\ndocs:\n  root: docs\n  index: docs/README.md\n  api:\n{}stories: []\n",
        if entries.is_empty() {
            "    []\n".to_string()
        } else {
            entries
        },
        if rollback.is_empty() { "''" } else { rollback },
        if pages.is_empty() {
            "    []\n".to_string()
        } else {
            pages
        }
    )
}

fn seeded(extra: &str, release_yaml: &str) -> Project {
    let p = Project::new();
    p.write(".devforgeai/gates.toml", &gates(extra));
    p.write(".devforgeai/releases/v0.3.0.yaml", release_yaml);
    p.set_phase("release", "v0.3.0");
    p
}

const MANIFEST_CHECK: &str = "  [[gate.check]]\n  kind = \"deploy_manifest\"\n  id = \"release-manifest\"\n  path = \"releases/{id}.yaml\"\n  platform = \"\"\n";

/// A `docs_cover` check whose `api_symbols_command` prints one tab-separated
/// `<kind>\t<symbol>\t<path>` line per symbol.
///
/// The separator is the point: a line with no tab is not a symbol line, so a
/// bare `echo place_order` names nothing and the check has no universe to
/// cover. The two shells need different quoting to keep the tabs — `sh` would
/// split an unquoted line on them, and `cmd` has no `printf` — so the command
/// is chosen per platform rather than written once and silently wrong on one.
fn docs_cover_check(id: &str, min_ratio: &str, symbols: &[(&str, &str, &str)]) -> String {
    let command = if cfg!(windows) {
        let lines: Vec<String> = symbols
            .iter()
            .map(|(k, s, p)| format!("echo {k}\\t{s}\\t{p}"))
            .collect();
        lines.join(" & ")
    } else {
        let body: String = symbols
            .iter()
            .map(|(k, s, p)| format!("{k}\\t{s}\\t{p}\\n"))
            .collect();
        format!("printf '{body}'")
    };
    format!(
        "  [[gate.check]]\n  kind = \"docs_cover\"\n  id = \"{id}\"\n  \
         path = \"releases/{{id}}.yaml\"\n  min_ratio = {min_ratio}\n  \
         command = \"{command}\"\n"
    )
}

// ------------------------------------------------------------ platform none

#[test]
fn the_none_platform_wants_an_empty_manifest_list() {
    let p = seeded(MANIFEST_CHECK, &release("none", &[], "", &[]));
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "pass", "{v}");
}

#[test]
fn the_none_platform_refuses_a_listed_manifest() {
    let p = seeded(
        MANIFEST_CHECK,
        &release("none", &["deploy/kubernetes/deployment.yaml"], "", &[]),
    );
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "fail");
    assert!(
        reason_of(&v, "release-manifest").contains("DFA-E342"),
        "{v}"
    );
}

#[test]
fn the_none_platform_refuses_a_rollback_note() {
    let p = seeded(
        MANIFEST_CHECK,
        &release("none", &[], "deploy/ROLLBACK.md", &[]),
    );
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "fail");
}

// ------------------------------------------------------------- kubernetes

const K8S: &[&str] = &[
    "deploy/kubernetes/deployment.yaml",
    "deploy/kubernetes/service.yaml",
    "deploy/kubernetes/ingress.yaml",
    "deploy/kubernetes/kustomization.yaml",
];

const DEPLOYMENT: &str = "apiVersion: apps/v1\nkind: Deployment\nspec:\n  template:\n    spec:\n      containers:\n        - name: app\n          image: example/app:1.0\n          readinessProbe:\n            httpGet:\n              path: /healthz\n          livenessProbe:\n            httpGet:\n              path: /healthz\n          resources:\n            limits:\n              cpu: 500m\n";

fn k8s_project(extra_manifest: Option<(&str, &str)>) -> Project {
    let p = seeded(
        MANIFEST_CHECK,
        &release("kubernetes", K8S, "deploy/kubernetes/ROLLBACK.md", &[]),
    );
    p.write("deploy/kubernetes/deployment.yaml", DEPLOYMENT);
    p.write("deploy/kubernetes/service.yaml", "kind: Service\n");
    p.write("deploy/kubernetes/ingress.yaml", "kind: Ingress\n");
    p.write("deploy/kubernetes/kustomization.yaml", "resources: []\n");
    if let Some((path, body)) = extra_manifest {
        p.write(path, body);
    }
    p
}

#[test]
fn a_complete_kubernetes_set_passes() {
    let p = k8s_project(None);
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "pass", "{v}");
}

#[test]
fn an_absent_manifest_is_e342() {
    let p = k8s_project(None);
    std::fs::remove_file(p.root().join("deploy/kubernetes/service.yaml")).expect("remove");
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "fail");
    let reason = reason_of(&v, "release-manifest");
    assert!(reason.contains("DFA-E342"), "{reason}");
    assert!(reason.contains("is absent"), "{reason}");
}

#[test]
fn a_manifest_that_does_not_parse_is_e342() {
    let p = k8s_project(None);
    p.write("deploy/kubernetes/service.yaml", "kind: [unclosed\n");
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "fail");
    assert!(
        reason_of(&v, "release-manifest").contains("does not parse"),
        "{v}"
    );
}

#[test]
fn a_deployment_without_a_probe_is_e342() {
    let p = k8s_project(None);
    p.write(
        "deploy/kubernetes/deployment.yaml",
        &DEPLOYMENT.replace(
            "          readinessProbe:\n            httpGet:\n              path: /healthz\n",
            "",
        ),
    );
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "fail");
    assert!(
        reason_of(&v, "release-manifest").contains("readinessProbe"),
        "{v}"
    );
}

#[test]
fn a_missing_kubernetes_file_in_the_list_is_e342() {
    let p = seeded(
        MANIFEST_CHECK,
        &release(
            "kubernetes",
            &K8S[..3],
            "deploy/kubernetes/ROLLBACK.md",
            &[],
        ),
    );
    p.write("deploy/kubernetes/deployment.yaml", DEPLOYMENT);
    p.write("deploy/kubernetes/service.yaml", "kind: Service\n");
    p.write("deploy/kubernetes/ingress.yaml", "kind: Ingress\n");
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "fail");
    assert!(
        reason_of(&v, "release-manifest").contains("kustomization.yaml"),
        "{v}"
    );
}

#[test]
fn a_literal_secret_in_a_manifest_is_e343() {
    let p = k8s_project(None);
    p.write(
        "deploy/kubernetes/service.yaml",
        "kind: Service\napi_key: AKIA1234567890ABCDEF\n",
    );
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "fail");
    let reason = reason_of(&v, "release-manifest");
    assert!(reason.contains("DFA-E343"), "{reason}");
    assert!(reason.contains(":2"), "{reason}");
}

#[test]
fn a_reference_is_not_a_literal_secret() {
    let p = k8s_project(None);
    p.write(
        "deploy/kubernetes/service.yaml",
        "kind: Service\napi_key: ${APP_API_KEY}\npassword:\n  valueFrom:\n    secretKeyRef:\n      name: app\n",
    );
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "pass", "{v}");
}

// ---------------------------------------------------------------- compose

#[test]
fn a_complete_compose_set_passes() {
    let p = seeded(
        MANIFEST_CHECK,
        &release(
            "compose",
            &[
                "deploy/compose/docker-compose.yaml",
                "deploy/compose/env.example",
            ],
            "deploy/compose/ROLLBACK.md",
            &[],
        ),
    );
    p.write(
        "deploy/compose/docker-compose.yaml",
        "services:\n  app:\n    image: example/app:1.0\n    environment:\n      PORT: ${APP_PORT}\n",
    );
    p.write("deploy/compose/env.example", "APP_PORT=8080\n");
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "pass", "{v}");
}

#[test]
fn a_compose_reference_with_no_env_example_line_is_e342() {
    let p = seeded(
        MANIFEST_CHECK,
        &release(
            "compose",
            &[
                "deploy/compose/docker-compose.yaml",
                "deploy/compose/env.example",
            ],
            "deploy/compose/ROLLBACK.md",
            &[],
        ),
    );
    p.write(
        "deploy/compose/docker-compose.yaml",
        "services:\n  app:\n    image: example/app:1.0\n    environment:\n      PORT: ${APP_PORT}\n",
    );
    p.write("deploy/compose/env.example", "OTHER=1\n");
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "fail");
    assert!(
        reason_of(&v, "release-manifest").contains("APP_PORT="),
        "{v}"
    );
}

#[test]
fn a_compose_service_with_neither_image_nor_build_is_e342() {
    let p = seeded(
        MANIFEST_CHECK,
        &release(
            "compose",
            &[
                "deploy/compose/docker-compose.yaml",
                "deploy/compose/env.example",
            ],
            "deploy/compose/ROLLBACK.md",
            &[],
        ),
    );
    p.write(
        "deploy/compose/docker-compose.yaml",
        "services:\n  app:\n    ports: ['8080:8080']\n",
    );
    p.write("deploy/compose/env.example", "APP_PORT=8080\n");
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "fail");
    assert!(
        reason_of(&v, "release-manifest").contains("has no image"),
        "{v}"
    );
}

// ------------------------------------------------------------------- vps

#[test]
fn a_complete_vps_set_passes() {
    let p = seeded(
        MANIFEST_CHECK,
        &release(
            "vps",
            &["deploy/vps/deploy.sh", "deploy/vps/app.service"],
            "deploy/vps/ROLLBACK.md",
            &[],
        ),
    );
    p.write("deploy/vps/deploy.sh", "#!/bin/sh\nset -e\n");
    p.write(
        "deploy/vps/app.service",
        "[Service]\nExecStart=/usr/bin/app\n",
    );
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "pass", "{v}");
}

#[test]
fn a_vps_script_without_a_shebang_is_e342() {
    let p = seeded(
        MANIFEST_CHECK,
        &release(
            "vps",
            &["deploy/vps/deploy.sh", "deploy/vps/app.service"],
            "deploy/vps/ROLLBACK.md",
            &[],
        ),
    );
    p.write("deploy/vps/deploy.sh", "set -e\n");
    p.write(
        "deploy/vps/app.service",
        "[Service]\nExecStart=/usr/bin/app\n",
    );
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "fail");
    assert!(reason_of(&v, "release-manifest").contains("shebang"), "{v}");
}

#[test]
fn a_vps_unit_without_an_execstart_is_e342() {
    let p = seeded(
        MANIFEST_CHECK,
        &release(
            "vps",
            &["deploy/vps/deploy.sh", "deploy/vps/app.service"],
            "deploy/vps/ROLLBACK.md",
            &[],
        ),
    );
    p.write("deploy/vps/deploy.sh", "#!/usr/bin/env sh\nset -e\n");
    p.write("deploy/vps/app.service", "[Service]\nType=simple\n");
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "fail");
    assert!(
        reason_of(&v, "release-manifest").contains("ExecStart"),
        "{v}"
    );
}

// -------------------------------------------------------- github-actions

#[test]
fn a_github_actions_workflow_passes_when_it_parses() {
    let p = seeded(MANIFEST_CHECK, &release("github-actions", &[], "", &[]));
    p.write(
        ".github/workflows/deploy.yml",
        "name: deploy\non:\n  push:\n    branches: [main]\njobs:\n  deploy:\n    runs-on: ubuntu-latest\n    steps: []\n",
    );
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "pass", "{v}");
}

#[test]
fn an_absent_github_actions_workflow_is_e342() {
    let p = seeded(MANIFEST_CHECK, &release("github-actions", &[], "", &[]));
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "fail");
    assert!(
        reason_of(&v, "release-manifest").contains("is absent"),
        "{v}"
    );
}

// -------------------------------------------------------------- docs_cover

const DOCS_CHECK: &str = "  [[gate.check]]\n  kind = \"docs_cover\"\n  id = \"release-docs-cover\"\n  path = \"releases/{id}.yaml\"\n  min_ratio = 1.0\n";

#[test]
fn docs_cover_skips_with_no_symbols_command() {
    let p = seeded(
        DOCS_CHECK,
        &release("none", &[], "", &["docs/api/index.md"]),
    );
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-docs-cover"), "skip");
    assert_eq!(
        reason_of(&v, "release-docs-cover"),
        "no_api_symbols_command"
    );
}

#[test]
fn docs_cover_passes_when_every_symbol_has_a_heading() {
    let p = seeded(
        &docs_cover_check(
            "release-docs-cover",
            "1.0",
            &[("fn", "place_order", "src/orders.rs")],
        ),
        &release("none", &[], "", &["docs/api/index.md"]),
    );
    p.write(
        "docs/api/index.md",
        "# API\n\n### place_order\n\nPlaces an order.\n",
    );
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-docs-cover"), "pass", "{v}");
}

#[test]
fn docs_cover_fails_with_e344_and_names_the_missing_symbols() {
    let p = seeded(
        &docs_cover_check(
            "release-docs-cover",
            "1.0",
            &[("fn", "cancel_order", "src/orders.rs")],
        ),
        &release("none", &[], "", &["docs/api/index.md"]),
    );
    p.write("docs/api/index.md", "# API\n\n### place_order\n");
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-docs-cover"), "fail");
    let reason = reason_of(&v, "release-docs-cover");
    assert!(reason.contains("DFA-E344"), "{reason}");
    assert!(reason.contains("cancel_order"), "{reason}");
}

#[test]
fn docs_cover_honours_a_lower_min_ratio() {
    let p = seeded(
        &docs_cover_check(
            "release-docs-cover",
            "0.5",
            &[
                ("fn", "place_order", "src/orders.rs"),
                ("fn", "cancel_order", "src/orders.rs"),
            ],
        ),
        &release("none", &[], "", &["docs/api/index.md"]),
    );
    p.write("docs/api/index.md", "# API\n\n### place_order\n");
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-docs-cover"), "pass", "{v}");
}

#[test]
fn the_release_platform_config_supplies_the_platform() {
    let p = seeded(MANIFEST_CHECK, &release("", &[], "", &[]));
    let mut cfg = devforgeai::config::load(p.root()).expect("config");
    cfg.release.platform = "none".into();
    p.write_config(&cfg);
    let v = gate_json(&p);
    assert_eq!(status_of(&v, "release-manifest"), "pass", "{v}");
}
