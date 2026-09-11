# Round 1 — actors (workflow step 6)

question: "Which of these roles uses or is affected by <slug>?"
header: "Actors"
multiSelect: true
options, one per candidate from persona-mapper, up to four:
  label: <persona name>
  description: <persona description> + " Wants: " + <persona goal>

Routing: the selected candidates become `personas[]`. A candidate left
unselected is dropped from the run. A reply naming a role outside the options
adds one persona record with that role as `name`, and the model writes its
`description` and `goal` from the same reply.
An empty selection re-asks this question with the same options.

# Round 2 — outcomes (workflow step 7)

question: "Which of these has to be true for <slug> to be worth building?"
header: "Outcomes"
multiSelect: true
options, three to four drafted from the flow statements or the description:
  label: <outcome, up to 40 characters>
  description: <the same outcome as one sentence with one number and one unit>

Routing: step 11 emits one epic per selected outcome, and that outcome is the
epic's `success_metric`. An empty selection re-asks this question with the same
options.

# Round 3 — boundaries (workflow step 8)

question: "Which of these is outside <slug> for this round?"
header: "Boundaries"
multiSelect: true
options, three to four drafted from the flow statements or the description:
  label: <exclusion, up to 40 characters>
  description: <the same exclusion as one sentence>

Routing: step 11 places each selected exclusion on exactly one epic as an
`out_of_scope` entry. An epic drawing no exclusion, and every epic when the
selection is empty, takes the single entry
"Nothing was named out of scope at discovery."

Rounds 1, 2, and 3 run at entry points A and B. Entry point C skips all three.

# Round 4 — acceptance (workflow step 13)

question: "Accept these <n> epics and <m> requirements as the requirement set for <slug>?"
header: "Accept epics"
multiSelect: false
options:
  label: "Accept"
  description: "Freeze this set. Constitute and Plan read it next."
  label: "Regroup epics"
  description: "Keep every requirement and its text. Rebuild only the epic grouping."
  label: "Redraft requirements"
  description: "Name the REQ ids to rewrite in your reply. Every other id keeps its text."

Routing:
  "Accept" -> workflow step 14.
  "Regroup epics" -> workflow step 11 with the epic block cleared, then this
    question again.
  "Redraft requirements" -> workflow step 9 for the REQ ids in the reply, then
    this question again. A reply carrying no REQ id re-asks this question
    unchanged.
