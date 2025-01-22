
use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Calculation(Vec<u8>);

impl Calculation {
    pub fn eval() -> String {
        todo!()
    }
}
