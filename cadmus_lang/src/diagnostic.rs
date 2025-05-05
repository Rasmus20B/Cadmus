use crate::token::SourceLoc;

pub enum DiagnosticCategory {
    BadIDFormat,
}

pub struct Diagnostic {
    category: DiagnosticCategory,
    location: SourceLoc,
    snippet: String,
}
