# Phenotype for Structured AI Prompts

This example demonstrates how Phenotyper can generate structured system
prompts for LLM applications.

```pht
namespace aivolution/ai/prompt;
```

## Prompt Sections

Each section of a prompt has a heading and body text. The plural companion
`PromptSections` collects sections for joining.

```pht
PromptSection plural PromptSections:
    heading: required string,
    body: required string,
    "## ", @(heading), "\n\n", @(body)
;
```

## System Prompt

A system prompt combines a role declaration, a set of instruction sections,
and an optional context block.

```pht
SystemPrompt:
    role: required string,
    instructions: required PromptSections,
    context: optional string,
    "# System Prompt\n\n",
    "You are ", @(role), ".\n\n",
    @join(instructions, "\n\n"),
    @ifset(context) { "\n\n---\n\n", @(context) }
;
```

When `context` is `None`, the trailing separator and context block are
omitted entirely. When present, they render after the instructions.
