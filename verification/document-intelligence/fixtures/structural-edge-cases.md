---
title: Structural Edge Cases
status: experimental
owners:
  - documentation-platform
tags: [markdown, parser, benchmark]
---

This preamble belongs to the document root rather than to the first heading.
It contains heading-like text such as `## not a heading` and an explicit link
to the [authority decision](linked-decision.md#authority-boundary).

Setext Document Section
=======================

The setext heading is a real section. The thematic break below is not front
matter because it does not begin the file.

---

## Lifecycle

Lifecycle introduction.

### Recovery

The lifecycle recovery procedure restores the last verified checkpoint.

##### Skipped Level

This heading deliberately skips level four. It remains a child according to
the chosen deterministic ownership rule; the parser must not invent a missing
heading.

## Operations

### Recovery

This duplicate title describes operator recovery, not lifecycle recovery.
Title-only edit addressing is therefore ambiguous.

> ## Quoted Heading
>
> This heading is nested inside a blockquote and must retain that block
> ownership rather than becoming a top-level section accidentally.

- ### List-Owned Heading

  This heading is nested inside a list item.

```markdown
## Fenced Fake Heading

[not-a-document-link](missing-inside-fence.md)
```

<h2>Raw HTML Heading</h2>

The raw HTML element must not silently become a Markdown ATX section.

| Signal | Expected interpretation |
|---|---|
| `##` in code | literal code content |
| duplicate heading | separate structural occurrence |
| explicit link | deterministic relationship |

- [x] parser fixture defined
- [ ] parser candidate selected

Reference-style links are structural too: [decision][authority].

[authority]: linked-decision.md#authority-boundary "Authority decision"
