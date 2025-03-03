
use super::parser::Token;

pub enum Command {
    Run { file_name: String, test_name: String },
    Show { },
    Reset, 
    Quit,
}
