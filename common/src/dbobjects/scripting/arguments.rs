

use crate::dbobjects::{reference::*, calculation::Calculation};

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldSelection {
    FromList(FieldReference),
    ByCalculation(Calculation),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScriptSelection {
    FromList(ScriptReference),
    ByCalculation(Calculation),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecordSelection {
    First,
    Last,
    Next,
    Previous,
    ByCalc(Calculation),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LayoutSelection {
    Current,
    FromList(LayoutReference),
    NameByCalculation(Calculation),
    NumberByCalculation(Calculation),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LayoutAnimation {
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WindowStyle {
    Document,
    FloatingDocument,
    Dialog,
    Card,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WindowOptions {
    Close,
    Minimize,
    Maximize,
    Resize,
    MenuBar,
    Toolbars,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Size {
    height: f64,
    width: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position {
    from_top: f64,
    from_left: f64,
}



