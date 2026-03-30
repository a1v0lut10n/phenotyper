# Phenotype for Configuration File Generation

This example demonstrates enum types, union type aliases, and nested
plural structures for generating configuration files.

```pht
namespace aivolution/format/config;
```

## Visibility Enum

A namespace-level enum defines the visibility levels for configuration
entries. Generated Rust renders the **original spelling** (e.g., `"public"`,
not `"Public"`).

```pht
type Visibility: [public, private, internal];
```

## Config Values (Union)

A union type alias allows configuration values to be strings, integers,
floats, or booleans.

```pht
type ConfigValue: {string, int64, real64, bool};
```

## Config Entries

Each entry combines a key, value, and visibility annotation.

```pht
ConfigEntry plural ConfigEntries:
    key: required string,
    value: required ConfigValue,
    visibility: required Visibility,
    @(key), " = ", @(value), " [", @(visibility), "]"
;
```

## Config Sections

Sections group entries under a named heading.

```pht
ConfigSection plural ConfigSections:
    name: required string,
    entries: required ConfigEntries,
    "[", @(name), "]\n",
    @join(entries, "\n")
;
```

## Config File

A complete config file with a title and multiple sections.

```pht
ConfigFile:
    title: required string,
    sections: required ConfigSections,
    "# ", @(title), "\n\n",
    @join(sections, "\n\n")
;
```
