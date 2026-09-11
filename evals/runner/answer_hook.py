#!/usr/bin/env python3
"""PreToolUse handler for AskUserQuestion during a DevForgeAI eval.

Reads the hook payload on stdin, matches each question against the case's
answers map, and allows the call with an ``updatedInput`` that echoes
``questions`` and adds ``answers``. A question with no configured answer is
denied, naming the header, so the case fails visibly instead of stalling.

Invocation, exec form, from the workspace ``.claude/settings.json``::

    {"type": "command", "command": "<python>", "args": ["<this file>",
     "<answers json>"], "timeout": 20}

The runner copies this file into the workspace ``.claude/`` directory rather
than pointing at the framework tree, so the workspace stays the measurement
boundary (AUDIT-5 EVL-007). The answers file path arrives in ``argv[1]``; the
record of what was matched is written beside it as ``eval-answers-used.json``,
which ``asked_decision`` reads to tell a real AskUserQuestion call from a phrase
the model typed (EVL-005).

The case's ``answers`` map is keyed by the question's ``header`` (preferred, the
headers are fixed in the skill templates), or by any substring of the question
text, or by ``"*"`` as the case's default for a question it did not name. A
value is an option ``label``, a list of labels for a ``multiSelect`` question,
or a positional selector — ``@first``, ``@last``, ``@n`` (1-based) or ``@all`` —
for a question whose option labels are drafted at run time.

Contract: ``specs/ANTHROPIC-GUIDANCE.md`` section 4 (AskUserQuestion headless)
and section 1 (PreToolUse decisions live in ``hookSpecificOutput``;
``updatedInput`` replaces the whole input object). Standard library only.
"""

import json
import os
import sys

USED_NAME = "eval-answers-used.json"


def emit(payload):
    sys.stdout.write(json.dumps(payload))
    sys.exit(0)


def allow(questions, answers):
    emit({"hookSpecificOutput": {
        "hookEventName": "PreToolUse",
        "permissionDecision": "allow",
        "updatedInput": {"questions": questions, "answers": answers}}})


def deny(reason):
    emit({"hookSpecificOutput": {
        "hookEventName": "PreToolUse",
        "permissionDecision": "deny",
        "permissionDecisionReason": reason}})


def load(path, fallback):
    try:
        with open(path, "r", encoding="utf-8") as handle:
            return json.load(handle)
    except (OSError, ValueError):
        return fallback


def key_of(question):
    """Match on header first, then on the question text."""
    return (str(question.get("header") or "").strip(),
            str(question.get("question") or "").strip())


def lookup(configured, header, text):
    lowered = {str(k).strip().lower(): v for k, v in configured.items()}
    for candidate in (header, text):
        if candidate and candidate.strip().lower() in lowered:
            return lowered[candidate.strip().lower()]
    # A question asked under a header the case spells differently still
    # resolves when one configured key is a substring of the question text.
    for k, v in lowered.items():
        if k == "*":
            continue
        if k and text and k in text.strip().lower():
            return v
    # "*" is the case's default for a question it did not name. A workflow that
    # crosses several questions but turns on only one of them names that one and
    # leaves the rest to the default.
    return lowered.get("*")


def resolve_selectors(wanted, allowed):
    """Replace a positional selector with the label the question offered.

    A question whose option labels are drafted at run time — the Explore holder
    segments, the Discover actor candidates — cannot be answered by label from a
    case line. ``@first``, ``@last``, ``@n`` (1-based) and ``@all`` pick by
    position instead. A label that is not a selector passes through unchanged.
    """
    out = []
    for one in wanted:
        token = str(one).strip().lower()
        if not token.startswith("@"):
            out.append(one)
            continue
        if not allowed:
            return None, ("selector %r needs the question to offer options, "
                          "and it offered none" % one)
        if token == "@all":
            out.extend(allowed)
            continue
        if token == "@first":
            index = 1
        elif token == "@last":
            index = len(allowed)
        else:
            try:
                index = int(token[1:])
            except ValueError:
                return None, "selector %r is not @first, @last, @n or @all" % one
        if not 1 <= index <= len(allowed):
            return None, ("selector %r names option %d and the question offers "
                          "%d" % (one, index, len(allowed)))
        out.append(allowed[index - 1])
    seen, unique = set(), []
    for one in out:
        if one not in seen:
            seen.add(one)
            unique.append(one)
    return unique, ""


def labels_of(question):
    out = []
    for option in question.get("options") or []:
        if isinstance(option, dict) and option.get("label") is not None:
            out.append(str(option["label"]))
        elif isinstance(option, str):
            out.append(option)
    return out


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    answers_path = argv[0] if argv else os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "eval-answers.json")
    used_path = os.path.join(os.path.dirname(os.path.abspath(answers_path)),
                             USED_NAME)
    try:
        payload = json.load(sys.stdin)
    except ValueError as exc:
        deny("eval hook: the payload on stdin is not JSON: %s" % exc)
    tool_input = payload.get("tool_input") or {}
    questions = tool_input.get("questions") or []
    if not isinstance(questions, list) or not questions:
        deny("eval hook: AskUserQuestion carried no questions list")

    configured = load(answers_path, {})
    if not isinstance(configured, dict) or not configured:
        deny("eval hook: this case configures no answers; "
             "AskUserQuestion is not available in a headless eval")

    answers, matched = {}, []
    for question in questions:
        if not isinstance(question, dict):
            deny("eval hook: a questions[] entry is not an object")
        header, text = key_of(question)
        chosen = lookup(configured, header, text)
        if chosen is None:
            deny("eval hook: no answer is configured for question header %r "
                 "(question %r); add it to the case's answers map"
                 % (header, text))
        allowed = labels_of(question)
        wanted = chosen if isinstance(chosen, list) else [chosen]
        wanted, problem = resolve_selectors(wanted, allowed)
        if problem:
            deny("eval hook: header %r: %s" % (header, problem))
        if question.get("multiSelect") is not True and len(wanted) != 1:
            deny("eval hook: header %r is single-select and the case "
                 "configures %d labels" % (header, len(wanted)))
        unknown = [one for one in wanted if allowed and str(one) not in allowed]
        if unknown:
            deny("eval hook: header %r configures label(s) %s, and the options "
                 "offered are %s" % (header, unknown, allowed))
        answer = wanted if question.get("multiSelect") is True else wanted[0]
        # Guidance section 4 describes `answers` as mapping the question text to
        # the chosen label; the templates key on `header`. Both keys carry the
        # same value, so whichever the tool reads finds it.
        answers[text or header] = answer
        if header:
            answers[header] = answer
        matched.append({"header": header, "question": text, "answer": answer})

    try:
        with open(used_path, "w", encoding="utf-8") as handle:
            json.dump(load(used_path, []) + matched, handle, ensure_ascii=False)
    except OSError:
        pass

    allow(questions, answers)


if __name__ == "__main__":
    main()
