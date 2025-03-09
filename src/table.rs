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
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub cells: Vec<String>,
}
