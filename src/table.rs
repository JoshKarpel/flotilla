use std::fmt::{Display, Error, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceTable {
    pub column_definitions: Vec<ColumnDefinition>,
    pub rows: Vec<Row>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnDefinition {
    pub name: String,
    r#type: String,
    description: String,
    format: String,
    priority: u8, // TODO: respect priority
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CellValue {
    String(String),
    Number(f64),
}

impl Display for CellValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}",
            match self {
                CellValue::String(s) => s.clone(),
                CellValue::Number(n) => n.to_string(),
            }
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub cells: Vec<CellValue>,
}
