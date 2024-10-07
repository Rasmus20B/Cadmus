set db_root to (do shell script "pwd") 
tell application "FileMaker Pro"
  activate
  open db_root & "/input/relation.fmp12"
  tell table "blank"
    create field "addition"
  end tell
end tell

