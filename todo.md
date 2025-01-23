# Cadmus TODO

## Cadlang compilation process.
- When compiling to fmp12, we open all externals and resolve all references eagerly.

- When compiling for use with emulator, the file is compiled, external
  references are stored, but the externals themselves are not compiled until
  they are needed.
    - For the case of table occurrences, their data sources and base tables are
      stored as strings until their actual references are needed. 

      For example, we switch to a layout that looks at table occurrence "a"
      that has a relationship with table occurrence "b". "b" is stored in
      another file.

      1. on initial compilation, "a" has stored "b"'s information as strings. It
         sees this other table as { data_source = "b.cad", table = "b" }.

      2. Because "a" is currently in context, we now have to load b.cad, and
         resolve this "b" table occurrence to instead look like { data_source =
         2, table = 1 }.
