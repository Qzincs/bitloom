use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct EnumVariant {
    pub value: i128,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum FieldType {
    Fixed(i128),
    Enum(Vec<EnumVariant>),
    Range {
        min: i128,
        max: i128,
        is_signed: bool,  
    },
    Expr(String),  // rhai script to compute the value
    Input,         // data provided by user input
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum FieldLength {
    /// Fixed length in bits
    Fixed(u32),
    Variable,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct FieldRule {
    pub id: String,
    pub name: Option<String>,
    pub field_type: FieldType,
    pub length: FieldLength,
    pub description: Option<String>,
}

impl FieldRule {
    pub fn new(id: &str, field_type: FieldType, length: FieldLength) -> Self {
        Self {
            id: id.to_string(),
            name: None,
            field_type,
            length,
            description: None,
        }
    }
}

impl Default for FieldRule {
    fn default() -> Self {
        Self {
            id: "new_field".to_string(),
            name: None,
            field_type: FieldType::Fixed(0),
            length: FieldLength::Fixed(8),
            description: None,
        }
    }
}

/// An instance of a field in a protocol message
pub struct Field {
    pub rule_id: String,
    pub value: Vec<u8>,
    pub ignore_rules: bool,
}


#[cfg(test)]
mod tests
{
    use super::*;
    #[test]
    fn test_field_rule_creation() {
        let default_field = FieldRule::default();
        assert_eq!(default_field.id, "new_field");
        assert_eq!(default_field.length, FieldLength::Fixed(8));

        let custom_field = FieldRule::new("version", FieldType::Fixed(4), FieldLength::Fixed(8));
        assert_eq!(custom_field.id, "version");
        assert_eq!(custom_field.field_type, FieldType::Fixed(4));
    }
}
