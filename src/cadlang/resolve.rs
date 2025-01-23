
use super::staging::Stage;

pub fn construct_external_references(stage: &mut Stage) -> Result<(), ()> {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use std::path::Path;

    use super::*;
    use crate::cadlang::{lexer, parser};
    #[test]
    fn single_file_sol() {
        let code = read_to_string(Path::new("./test_data/cad_files/initial.cad")).unwrap();

        let stage = parser::parse(&lexer::lex(&code).unwrap()).unwrap();
        println!("HETJHEJRHEJ");

        for (i, occ) in stage.table_occurrences {
            println!("{:?}", occ);
        }

    }
}
