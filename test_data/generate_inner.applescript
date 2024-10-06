set p to (do shell script "pwd") & "/input/relation.fmp12"
tell application "FileMaker Pro"
  activate
  open p
  tell table "blank"
    count fields
    count records
    cell "PrimaryKey"
  end tell
end tell

