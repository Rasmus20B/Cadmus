tell application "FileMaker Pro"
  activate
  open "/Users/merimak/prog/cadmus/test_data/input/blank_table_change.fmp12"
  tell table "othertable"
    count fields
    count records
    cell "PrimaryKey"
  end tell
end tell
