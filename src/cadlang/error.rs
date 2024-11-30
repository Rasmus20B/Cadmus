use core::fmt;
use super::{parser::FMObjType, token::{Token, TokenType}};

#[derive(Debug, PartialEq, Eq)]
pub enum CompileErr {
    UnexpectedToken { token: Token, expected: Vec<TokenType>},
    RelationCriteria { token: Token }, // criteria must have uniform tables.
    UnknownTable { token: Token },
    UnknownTableOccurrence { token: Token },
    UnknownField { token: Token },
    InvalidAssert { token: Token }, // Asserts can only be used in tests
    MissingAttribute { base_object: String, construct: String, specifier: String },
    UnimplementedLanguageFeauture { feature: String, token: Token },
    UndefinedReference { construct: FMObjType, token: Token },
    UnexpectedEOF,
}

impl<'a> fmt::Display for CompileErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken { token, expected } => {
                let val1 = if token.ttype == TokenType::Identifier {
                    format!("{}: \"{}\"", token.ttype, token.value)
                } else {
                    format!("\"{}\"", token.ttype)
                };
                if expected.len() > 1 {
                    write!(f, "Unexpected {} @ {},{}. Expected one of: {:?}", 
                        val1,
                        token.location.line,
                        token.location.column,
                        expected.iter().map(|t| t.to_string()).collect::<Vec<_>>())
                } else {
                    write!(f, "Unexpected {} @ {},{}. Expected: {:?}", 
                        val1,
                        token.location.line,
                        token.location.column,
                        expected.iter().map(|t| t.to_string()).collect::<Vec<_>>())
                }
            }
            Self::RelationCriteria { token } => {
                write!(f, "Found non-matching table \"{}\" reference in relation criteria. @ {}, {}", 
                    token.value,
                    token.location.line,
                    token.location.column)
            }
            Self::UnknownTable { token } => {
                write!(f, "Invalid reference to table: {} @ {}, {}", 
                    token.value,
                    token.location.line,
                    token.location.column)
            }
            Self::UnknownTableOccurrence { token } => {
                write!(f, "Invalid reference to table occurrence: {} @ {}, {}", 
                    token.value,
                    token.location.line,
                    token.location.column)
            }
            Self::UnknownField { token } => {
                write!(f, "Invalid reference to field: {} @ {}, {}", 
                    token.value,
                    token.location.line,
                    token.location.column)
            }
            Self::MissingAttribute { base_object, construct, specifier } => {
                write!(f, "Missing attribute {} for {} in {}", specifier, construct, base_object)
            }
            Self::UnimplementedLanguageFeauture { feature, token } => {
                write!(f, "Unimplemented language feature: {} used @ {},{}",
                    feature,
                    token.location.line,
                    token.location.column)
            }
            Self::UndefinedReference { construct, token } => {
                write!(f, "Undefined reference to {} \"{}\"", construct, token.value)
            }
            _ => write!(f, "nah not compiling.")
        }
    }

}
