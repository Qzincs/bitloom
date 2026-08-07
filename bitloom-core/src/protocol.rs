use crate::field::{Field, FieldLength, FieldRule};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Default)]
pub enum Endianness {
    #[default]
    Big,
    Little,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ProtocolLength {
    /// Fixed length in bits
    Fixed(u32),
    /// Variable length; the value denotes the fixed prefix length in bits
    Variable(u32),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Protocol {
    pub id: String,
    pub remark: Option<String>,
    pub endianness: Endianness,
    pub fields: Vec<FieldRule>,
    pub length: ProtocolLength,
    pub description: Option<String>,
    pub metadata: HashMap<String, String>,
    pub parent_id: Option<String>,                 // parent protocol ID
    pub parent_constraints: HashMap<String, i128>, // (field_id, value): constraints on parent fields for this subprotocol to apply
}

impl Protocol {
    pub fn new(
        id: &str,
        remark: Option<String>,
        endianness: Endianness,
        parent_id: Option<String>,
    ) -> Self {
        Self {
            id: id.to_string(),
            remark,
            endianness,
            fields: Vec::new(),
            length: ProtocolLength::Fixed(0),
            description: None,
            metadata: HashMap::new(),
            parent_id,
            parent_constraints: HashMap::new(),
        }
    }

    pub fn update_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    pub fn add_field(&mut self, field_rule: FieldRule) -> Result<(), String> {
        if self.fields.iter().any(|f| f.id == field_rule.id) {
            return Err(format!(
                "Field with ID '{}' already exists in protocol '{}'",
                field_rule.id, self.id
            ));
        }

        if let Some(last_field) = self.fields.last() {
            if let FieldLength::Variable = last_field.length {
                return Err(format!(
                    "Cannot add field '{}' after variable length field '{}' in protocol '{}'",
                    field_rule.id, last_field.id, self.id
                ));
            }
        }

        self.fields.push(field_rule);
        self.calculate_length();
        Ok(())
    }

    pub fn remove_field(&mut self, field_id: &str) -> Result<(), String> {
        let old_len = self.fields.len();
        self.fields.retain(|f| f.id != field_id);

        if self.fields.len() == old_len {
            return Err(format!(
                "Field with ID '{}' not found in protocol '{}'",
                field_id, self.id
            ));
        }

        self.calculate_length();
        Ok(())
    }

    pub fn move_field(&mut self, field_id: &str, new_index: usize) -> Result<(), String> {
        let Some(old_index) = self.fields.iter().position(|f| f.id == field_id) else {
            return Err(format!(
                "Field with ID '{}' not found in protocol '{}'",
                field_id, self.id
            ));
        };

        let mut fields = self.fields.clone();
        let field = fields.remove(old_index);
        let new_index = new_index.min(fields.len()); // ensure new_index is within bounds
        fields.insert(new_index, field);

        Self::validate_field_layout(&fields, &self.id)?;
        self.fields = fields;

        Ok(())
    }

    pub fn update_field_id(&mut self, old_id: &str, new_id: &str) -> Result<(), String> {
        if old_id == new_id {
            return Ok(()); // no change needed
        }

        if self.fields.iter().any(|f| f.id == new_id) {
            return Err(format!("Field with ID '{}' already exists", new_id));
        }

        if let Some(field) = self.fields.iter_mut().find(|f| f.id == old_id) {
            field.id = new_id.to_string();
            Ok(())
        } else {
            Err(format!("Field with ID '{}' does not exist", old_id))
        }
    }

    pub fn edit_field<F>(&mut self, field_id: &str, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut FieldRule) -> Result<(), String>,
    {
        let Some(index) = self.fields.iter().position(|f| f.id == field_id) else {
            return Err(format!(
                "Field with ID '{}' not found in protocol '{}'",
                field_id, self.id
            ));
        };

        let mut fields = self.fields.clone();

        {
            let field = &mut fields[index];
            if let Err(error) = f(field) {
                return Err(error);
            }

            // cannot change field ID through this method
            if field.id != self.fields[index].id {
                return Err(
                    "Field ID cannot be changed through edit_field; use update_field_id instead"
                        .to_string(),
                );
            }
        }

        Self::validate_field_layout(&fields, &self.id)?;

        let length_changed = fields[index].length != self.fields[index].length;
        self.fields = fields;
        if length_changed {
            self.calculate_length();
        }
        Ok(())
    }

    pub fn set_parent_constraint(&mut self, field_id: &str, value: i128) {
        // TODO: validate that field_id exists in parent protocol and value is valid for that field
        self.parent_constraints.insert(field_id.to_string(), value);
    }

    /// Calculate the total length of the protocol based on its fields.
    /// If any field has variable length, the protocol length is variable.
    /// Must be called after any change to the fields to keep the protocol length up to date.
    fn calculate_length(&mut self) {
        let mut total_fixed_bits = 0;
        for field in &self.fields {
            match field.length {
                FieldLength::Fixed(bits) => total_fixed_bits += bits,
                // variable field is always at the end
                FieldLength::Variable => {
                    self.length = ProtocolLength::Variable(total_fixed_bits);
                    return;
                }
            }
        }
        self.length = ProtocolLength::Fixed(total_fixed_bits);
    }

    /// Validate the layout of the protocol's fields.
    fn validate_field_layout(fields: &[FieldRule], protocol_id: &str) -> Result<(), String> {
        let Some(index) = fields
            .iter()
            .position(|f| matches!(&f.length, FieldLength::Variable))
        else {
            return Ok(());
        };

        if index != fields.len() - 1 {
            return Err(format!(
                "Variable length field '{}' must be the last field in protocol '{}'",
                fields[index].id, protocol_id
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProtocolTreeNode {
    pub id: String,
    pub remark: Option<String>,
    pub children: Vec<ProtocolTreeNode>,
}

pub struct Packet {
    pub protocol_id: String,
    pub field_values: Vec<Field>,
}

impl Packet {
    pub fn new(protocol_id: &str, field_rules: Vec<FieldRule>) -> Self {
        Self {
            protocol_id: protocol_id.to_string(),
            field_values: field_rules
                .into_iter()
                .map(|rule| Field::new(&rule.id, vec![], false))
                .collect(),
        }
    }

    pub fn set_field_value(&mut self, index: usize, value: Vec<u8>) -> Result<(), String> {
        if let Some(field) = self.field_values.get_mut(index) {
            field.set_value(value);
            Ok(())
        } else {
            Err(format!("Field at index {} not found in packet", index))
        }
    }

    pub fn is_complete(&self) -> bool {
        self.field_values.iter().all(|f| !f.value.is_empty())
    }
}

#[cfg(test)]
#[path = "protocol_tests.rs"]
mod tests;
