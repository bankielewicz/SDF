Call 1, four questions in one AskUserQuestion invocation:

AskUserQuestion(questions=[
  { "question": "<brand name>: what should it feel like to use?",
    "header": "Tone", "multiSelect": false,
    "options": [
      { "label": "Plain",    "description": "Nothing decorative. Short labels, one accent colour, generous white space." },
      { "label": "Warm",     "description": "Rounded corners, warm neutrals, sentence-case labels that sound like a person." },
      { "label": "Precise",  "description": "Tight grid, cool neutrals, a mono face for numbers and identifiers." },
      { "label": "Bold",     "description": "High contrast, large display sizes, one saturated colour carrying the brand." } ] },
  { "question": "<brand name>: which colour direction?",
    "header": "Color", "multiSelect": false,
    "options": [
      { "label": "Cool neutral",  "description": "Grey-blue surfaces, one green or blue primary. Reads as software." },
      { "label": "Warm neutral",  "description": "Sand and stone surfaces, one terracotta or olive primary. Reads as paper." },
      { "label": "Mono",          "description": "Black, white, and one accent used only on the primary action." },
      { "label": "From sketch",   "description": "The palette in .devforgeai/explore/mockups/brand-sketch.json, adjusted to clear contrast." } ] },
  { "question": "<brand name>: which type pairing?",
    "header": "Type", "multiSelect": false,
    "options": [
      { "label": "One sans",       "description": "A single sans face at six sizes and two weights." },
      { "label": "Sans and mono",  "description": "A sans for prose, a mono for numbers, ids, and code." },
      { "label": "Sans and serif", "description": "A serif for headings, a sans for everything else." },
      { "label": "Humanist sans",  "description": "One humanist sans with wide apertures, at six sizes and two weights." } ] },
  { "question": "<brand name>: how much room between things?",
    "header": "Density", "multiSelect": false,
    "options": [
      { "label": "Compact",     "description": "Dense tables and lists. More rows on screen, smaller targets." },
      { "label": "Comfortable", "description": "The middle. 44px targets, one line of air between rows." },
      { "label": "Spacious",    "description": "Wide gutters and large targets. Fewer rows, easier to scan." } ] }
])

Call 2, one question:

AskUserQuestion(questions=[
  { "question": "<brand name>: what shape is the mark?",
    "header": "Logo", "multiSelect": false,
    "options": [
      { "label": "Wordmark",          "description": "The name drawn as paths, no symbol." },
      { "label": "Monogram",          "description": "The first letter or two in a shape, usable alone at 24px." },
      { "label": "Geometric mark",    "description": "An abstract shape, usable alone at 24px, name set beside it." },
      { "label": "Wordmark and mark", "description": "A symbol and the name, locked together with fixed clear space." } ] }
])

Answer-to-leaf mapping:

| Header | Label | Leaves it sets |
|---|---|---|
| Tone | Plain | radius.sm 0.125rem, radius.md 0.25rem, elevation.raised none, type.weight-bold 600 |
| Tone | Warm | radius.sm 0.375rem, radius.md 0.75rem, elevation.raised a 2px soft shadow, type.weight-bold 600 |
| Tone | Precise | radius.sm 0.125rem, radius.md 0.25rem, elevation.raised a 1px hairline shadow, type.weight-bold 700 |
| Tone | Bold | radius.sm 0.25rem, radius.md 0.5rem, elevation.raised a 4px shadow, type.weight-bold 800 |
| Color | Cool neutral | the 12 color leaves, bg and surface at hue 210 to 230, primary at hue 150 to 210 |
| Color | Warm neutral | the 12 color leaves, bg and surface at hue 30 to 50, primary at hue 10 to 30 or 60 to 90 |
| Color | Mono | the 12 color leaves, bg, surface, border, text and text-muted at chroma 0, primary the one saturated value |
| Color | From sketch | the 12 color leaves, seeded from brand-sketch.json palette, each adjusted until its row in ## Color clears its ratio |
| Type | One sans | family-sans set, family-mono set to the platform mono stack |
| Type | Sans and mono | family-sans set, family-mono set to a named mono face |
| Type | Sans and serif | family-sans set, family-mono set to the platform mono stack, size-xl and size-xxl carrying the serif face |
| Type | Humanist sans | family-sans set to a humanist face, family-mono set to the platform mono stack |
| Density | Compact | spacing.xs 0.125rem, sm 0.25rem, md 0.5rem, lg 1rem, xl 1.5rem, xxl 2.5rem |
| Density | Comfortable | spacing.xs 0.25rem, sm 0.5rem, md 1rem, lg 1.5rem, xl 2.5rem, xxl 4rem |
| Density | Spacious | spacing.xs 0.375rem, sm 0.75rem, md 1.5rem, lg 2.5rem, xl 4rem, xxl 6rem |
| Logo | any of the four | the viewBox content of .devforgeai/brand/logo.svg |
