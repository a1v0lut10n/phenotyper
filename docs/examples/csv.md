# Phenotype for the generation of comma-separated value (CSV) files

Code will be generated using the `aivolution/format/csv` namespace.

```pht
aivolution/format/csv:
```

## Scalar Values

A CSV field may contain any scalar value. We define a union type to capture
the set of allowable value types.

```pht
type ScalarValue: {int64, real64, string, date, time, datetime};
```

## CSV Fields

A CSV field value wraps a single scalar value. The plural companion
`CSVFieldValues` provides the collection type used by CSV lines.

```pht
CSVFieldValue plural CSVFieldValues:
    value: required ScalarValue,
    @(value)
;
```

## CSV Lines

A CSV line is a set of field values joined by a separator (typically a comma).
The plural companion `CSVLines` collects multiple lines.

```pht
CSVLine plural CSVLines:
    fields: required CSVFieldValues,
    separator: required string,
    @join(fields, separator)
;
```

## CSV Files

A CSV file consists of a header line and a set of data lines, each terminated
by a configurable end-of-line sequence.

```pht
CSVFile:
    header: required CSVLine,
    lines: required CSVLines,
    eol: required string,
    @(header), @eol(eol), @join(lines, eol)
;
```
