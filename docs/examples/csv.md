# Phenotype for the generation of comma-separated value (CSV) files

Code will be generating using the aivolution/format/csv namespace.

```pht
namespace aivolution/format/csv;
```

## CSV Records

A CSV record is essentially a value which will be output using a formatter suitable for its type.

```pht
CSVRecord plural CSVRecords:
    field: required {int64, real64, string, date, time, datetime},
        @(field)
;
```

## CSV Lines

A CSV line is a set of CSV records output as a line of CSV records separated by a specified separator.

```pht
CSVLine plural CSVLines:
    records: CSVRecords,
    separator: required string,
    @join(records, separator)
;     
```


## CSV Files

A CSV file consists of a header CSV line and a set of CSV lines with values, each line
separated with a specified line separator.

```pht
CSVFile plural CSVFiles:
    header: CSVLine,
    lines: CSVLines,
    separator: required string,
            
    @(header),
    @join(lines, @eol),
        @eol
;
```
