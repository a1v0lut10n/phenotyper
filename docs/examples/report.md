# Phenotype for Report Generation with Optional Fields

This example demonstrates optional fields (`@ifset`) and conditional
collection rendering (`@ifnotempty`) for generating markdown-style reports.

```pht
aivolution/format/report:
```

## Tags

Tags are simple string labels. The plural companion `Tags` collects them
for use in conditional rendering.

```pht
Tag plural Tags:
    label: required string,
    @(label)
;
```

## Report Lines

Each report line wraps a content string.

```pht
ReportLine plural ReportLines:
    content: required string,
    @(content)
;
```

## Report

The main report type demonstrates:

- **`@ifset(subtitle)`**: Renders the subtitle block only when present.
- **`@ifnotempty(tags)`**: Renders the tags section only when the
  `tags` collection is non-empty.
- **`@ifset(footer)`**: Renders a footer only when provided.

```pht
Report:
    title: required string,
    subtitle: optional string,
    lines: required ReportLines,
    tags: required Tags*,
    footer: optional string,
    "# ", @(title), "\n",
    @ifset(subtitle) { "## ", @(subtitle), "\n" },
    "\n",
    @join(lines, "\n"),
    @ifnotempty(tags) { "\n\n---\nTags: ", @join(tags, ", ") },
    @ifset(footer) { "\n\n---\n", @(footer) }
;
```
