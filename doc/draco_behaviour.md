# Draco Engine Behaviour Findings

## Schema changes

### Table Metadata
When a table name is changed: 
- The metadata storage directory's consistency counter located @ ["3", "16", "1"]  will increment by 1.
- The consistency counter located in the table's metadata directory located @ ["3", "16", "5", <table_id>] will increment by 1. 
- The consistency counter in the table occurrence directory ["3", "17", "1"] will increment by 1.
- The consistency counter for layout metadata located in ["4", "1"] will increment by 1.
- The consistency counter for top-level layout storage located at ["4", "5", "1"] will increment by 1. 
