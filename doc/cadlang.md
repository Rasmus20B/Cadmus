# Cadlang Spec

## Top level objects

#### Tables
``` 
table %<Integer> <identifier> = { [Field],... }
``` 

#### Fields
```
field %<Integer> <identifier> = {
    [Attribute],...
}
```

##### Attributes
```
datatype = [Text, Number, Date, Time, Timestamp, Container]
```
```
comment = <String>
```
##### Auto-Entry
```
serial = {
    generate = [on_creation, on_commit],
    next = <Integer>,
    increment = <Integer>,
}
```
```
lookup = {
    start = <table_name>,
    related = <field_reference>,
}
```

##### Validation
```
validate = [always, entry]
```
```
override = [true, false]
```
```
not_empty = [true, false]
```
```
required = [true, false]
```
```
unique = [true, false]
```
```
member_of = <valuelist_name>
```
```
range = {
    start = <Integer>,
    end = <Integer>,
}
```
```
validation_calc = <Calculation>
```
```
max_chars = <Integer>
```
```
validation_message = <String>
```

#### Table Occurrences
```
table_occurrence %<Integer> <identifier> : <table_name>
```

#### Relations
```
relation %<Integer> <identifier> = <field_reference> [==, !=, >, >=, <, <=, *] <field_reference>
```

#### Compound Relations
```
relation %<Integer> <identifier> = { 
    <field_reference> [==, !=, >, >=, <, <=, *] <field_reference>,
    <field_reference> [==, !=, >, >=, <, <=, *] <field_reference>,
    ...
}
```
Note: Compound relations must use same table occurrence pairs for each line.

#### Layouts

```
layout %<Integer> <identifier> : <table_occurrence_name> {
    [layout_object_definition]...
}
```
#### Scripts

```
script %<Integer> <identifier> = {
    [script_step]...
}
```

Note: Top level scripts cannot use the assert() script step. This is because it is not included in FileMaker.

#### Tests
```
test %<Integer> <identifier> = {
    [script_step]...
}
```

Note: The assert() script step is reserved for cadmus tests.
