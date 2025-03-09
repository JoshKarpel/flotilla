use std::fmt::{Display, Error, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceTable {
    pub column_definitions: Vec<ColumnDefinition>,
    pub rows: Vec<ResourceRow>,
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
pub enum ResourceRowCellValue {
    String(String),
    Number(f64),
}

impl Display for ResourceRowCellValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}",
            match self {
                ResourceRowCellValue::String(s) => s.clone(),
                ResourceRowCellValue::Number(n) => n.to_string(),
            }
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceRow {
    pub cells: Vec<ResourceRowCellValue>,
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(ResourceRowCellValue::String("hello".to_string()), "hello")]
    #[case(ResourceRowCellValue::String("1/2".to_string()), "1/2")]
    #[case(ResourceRowCellValue::Number(42.0), "42")]
    #[case(ResourceRowCellValue::Number(42.5), "42.5")]
    fn test_display_resource_row_cell_value(
        #[case] value: ResourceRowCellValue,
        #[case] expected: &str,
    ) {
        assert_eq!(value.to_string(), expected);
    }
}
