#!/usr/bin/env python3
"""A minimal stdio MCP server exposing one permission-prompt tool.

``claude -p`` has no permission host of its own, and without one it drops every
tool that would prompt — ``AskUserQuestion`` included — from the tool set, so the
eval's PreToolUse answer seed has nothing to intercept. ``--permission-prompt-tool
mcp__dfa_permissions__approve`` names a host; this file is the smallest one that
satisfies it.

The tool allows every request unchanged: the eval's own decisions live in the
workspace ``settings.json`` (``--permission-mode acceptEdits``, ``--allowedTools``
and the ``AskUserQuestion`` answer hook), and a second gate here would only mask
them. It returns the documented permission-prompt shape::

    {"behavior": "allow", "updatedInput": <the tool input unchanged>}

as the single text content block of the tool result.

Protocol: JSON-RPC 2.0 over stdio, one message per line. Standard library only,
no dependency on an MCP package. Run by the eval runner through ``--mcp-config``;
it is never started by hand.
"""

import json
import sys

SERVER = "dfa-permissions"
TOOL = "approve"
PROTOCOL = "2024-11-05"

TOOLS = [{
    "name": TOOL,
    "description": ("Approve a tool call during a DevForgeAI eval. Returns "
                    "behavior allow with the input unchanged."),
    "inputSchema": {
        "type": "object",
        "properties": {
            "tool_name": {"type": "string"},
            "input": {"type": "object"},
            "tool_use_id": {"type": "string"},
        },
        "required": ["tool_name", "input"],
    },
}]


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def reply(request_id, result):
    send({"jsonrpc": "2.0", "id": request_id, "result": result})


def handle(message):
    method = message.get("method")
    request_id = message.get("id")
    if method == "initialize":
        reply(request_id, {
            "protocolVersion": PROTOCOL,
            "capabilities": {"tools": {}},
            "serverInfo": {"name": SERVER, "version": "1.0.0"}})
    elif method in ("notifications/initialized", "initialized"):
        return
    elif method == "tools/list":
        reply(request_id, {"tools": TOOLS})
    elif method == "tools/call":
        params = message.get("params") or {}
        arguments = params.get("arguments") or {}
        decision = {"behavior": "allow",
                    "updatedInput": arguments.get("input") or {}}
        reply(request_id, {"content": [{"type": "text",
                                        "text": json.dumps(decision)}],
                           "isError": False})
    elif method in ("resources/list", "prompts/list"):
        key = method.split("/")[0]
        reply(request_id, {key: []})
    elif method == "ping":
        reply(request_id, {})
    elif request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id,
              "error": {"code": -32601, "message": "no method %r" % method}})


def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            message = json.loads(line)
        except ValueError:
            continue
        try:
            handle(message)
        except Exception as exc:  # a host must not die on one bad message
            if message.get("id") is not None:
                send({"jsonrpc": "2.0", "id": message["id"],
                      "error": {"code": -32603, "message": str(exc)}})


if __name__ == "__main__":
    main()
