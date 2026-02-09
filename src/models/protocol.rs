use super::field::{Field, FieldLength, FieldRule, FieldType};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, iter};

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
    pub name: Option<String>,
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
        name: Option<String>,
        endianness: Endianness,
        parent_id: Option<String>,
    ) -> Self {
        Self {
            id: id.to_string(),
            name,
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
        if let Some(pos) = self.fields.iter().position(|f| f.id == field_id) {
            let field = self.fields.remove(pos);
            let new_index = new_index.min(self.fields.len()); // ensure new_index is within bounds
            self.fields.insert(new_index, field);
            Ok(())
        } else {
            Err(format!(
                "Field with ID '{}' not found in protocol '{}'",
                field_id, self.id
            ))
        }
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
        // find the field to edit
        if let Some(field) = self.fields.iter_mut().find(|f| f.id == field_id) {
            let backup = field.clone();
            // attempt to apply the edit function
            if let Err(e) = f(field) {
                *field = backup; // revert applied changes
                return Err(e);
            }
            
            // cannot change field ID through this method
            if field.id != backup.id {
                *field = backup; // revert applied changes
                return Err(
                    "Field ID cannot be changed through edit_field; use update_field_id instead"
                        .to_string(),
                );
            }

            if field.length != backup.length {
                self.calculate_length(); // recalculate protocol length if field length changed
            }
            
            Ok(())
        } else {
            Err(format!(
                "Field with ID '{}' not found in protocol '{}'",
                field_id, self.id
            ))
        }
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
}

pub struct ProtocolRegistry {
    /// map from protocol ID to Protocol definition
    protocols: HashMap<String, Protocol>,
}

impl ProtocolRegistry {
    pub fn new() -> Self {
        Self {
            protocols: HashMap::new(),
        }
    }

    pub fn create_protocol(
        &mut self,
        id: &str,
        name: Option<String>,
        endianness: Endianness,
        parent_id: Option<String>,
    ) -> Result<(), String> {
        if self.protocols.contains_key(id) {
            return Err(format!("Protocol with ID '{}' already exists", id));
        }

        if let Some(pid) = &parent_id {
            if !self.protocols.contains_key(pid) {
                return Err(format!("Parent protocol with ID '{}' does not exist", pid));
            }
        }

        let protocol = Protocol::new(id, name, endianness, parent_id);
        self.protocols.insert(id.to_string(), protocol);
        Ok(())
    }

    /// Remove a protocol and all its subprotocols recursively
    pub fn remove_protocol(&mut self, protocol_id: &str) -> Result<(), String> {
        if !self.protocols.contains_key(protocol_id) {
            return Err(format!("Protocol with ID '{}' does not exist", protocol_id));
        }

        let mut to_remove = vec![protocol_id.to_string()];
        let mut i = 0;

        while i < to_remove.len() {
            let current_id = &to_remove[i];
            let children: Vec<String> = self
                .protocols
                .values()
                .filter(|p| p.parent_id.as_deref() == Some(current_id))
                .map(|p| p.id.clone())
                .collect();
            to_remove.extend(children);
            i += 1;
        }

        for id in to_remove {
            self.protocols.remove(&id);
        }
        Ok(())
    }

    /// Change the ID of a protocol, and update all references to it (e.g. parent_id in child protocols)
    pub fn update_protocol_id(&mut self, old_id: &str, new_id: &str) -> Result<(), String> {
        if old_id == new_id {
            return Ok(()); // no change needed
        }

        if self.protocols.contains_key(new_id) {
            return Err(format!("Protocol with ID '{}' already exists", new_id));
        }

        if let Some(mut proto) = self.protocols.remove(old_id) {
            proto.id = new_id.to_string();
            self.protocols.insert(new_id.to_string(), proto);

            // Update parent references in child protocols
            for p in self.protocols.values_mut() {
                if p.parent_id.as_deref() == Some(old_id) {
                    p.parent_id = Some(new_id.to_string());
                }
            }
            Ok(())
        } else {
            Err(format!("Protocol with ID '{}' does not exist", old_id))
        }
    }

    pub fn get_protocol(&self, protocol_id: &str) -> Option<&Protocol> {
        self.protocols.get(protocol_id)
    }

    /// Edits the properties of an existing protocol using the provided closure.
    ///
    /// ### Constraints
    /// - The protocol `id` cannot be modified within this closure, please use [`Self::update_protocol_id`] instead.
    /// - The `parent_id` is immutable after creation to
    ///   ensure the stability of the inheritance tree.
    pub fn edit_protocol<F>(&mut self, protocol_id: &str, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut Protocol) -> Result<(), String>,
    {
        if let Some(proto) = self.protocols.get_mut(protocol_id) {
            let backup = proto.clone();

            // attempt to apply the edit function
            if let Err(e) = f(proto) {
                *proto = backup; // revert applied changes
                return Err(e);
            }

            if proto.id != backup.id {
                *proto = backup;
                return Err("Protocol ID cannot be changed through edit_protocol; use rename_protocol instead".to_string());
            }

            if proto.parent_id != backup.parent_id {
                *proto = backup;
                return Err(
                    "Inheritance relationship (parent_id) is immutable after creation".to_string(),
                );
            }

            Ok(())
        } else {
            Err(format!("Protocol with ID '{}' does not exist", protocol_id))
        }
    }

    /// Get the full inheritance chain of a protocol, starting from the root ancestor down to the protocol itself.
    pub fn get_inheritance_chain(&self, protocol_id: &str) -> Vec<&Protocol> {
        let mut chain = Vec::new();
        let mut current_id = Some(protocol_id);

        while let Some(id) = current_id {
            if let Some(proto) = self.protocols.get(id) {
                chain.push(proto);
                current_id = proto.parent_id.as_deref();
            } else {
                break; // invalid parent reference, stop the chain
            }
        }

        chain.reverse(); // reverse to get from root to leaf
        chain
    }

    /// Calculate the total length of a protocol by summing the lengths of all fields in its inheritance chain.
    pub fn get_total_length(&self, protocol_id: &str) -> ProtocolLength {
        let mut total_fixed_bits = 0;

        let chain = self.get_inheritance_chain(protocol_id);
        for proto in chain {
            match proto.length {
                ProtocolLength::Fixed(bits) => total_fixed_bits += bits,
                ProtocolLength::Variable(bits) => {
                    return ProtocolLength::Variable(total_fixed_bits + bits);
                }
            }
        }
        ProtocolLength::Fixed(total_fixed_bits)
    }

    /// Flatten and resolve all fields from the inheritance chain of a protocol.
    pub fn resolve_fields(&self, protocol_id: &str) -> Result<Vec<FieldRule>, String> {
        let chain = self.get_inheritance_chain(protocol_id);
        if chain.is_empty() {
            return Err(format!("Protocol with ID '{}' does not exist", protocol_id));
        }

        let mut resolved_fields = Vec::new();
        for proto in chain {
            resolved_fields.extend(proto.fields.iter().cloned());
        }
        Ok(resolved_fields)
    }
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
mod tests {
    use super::*;

    #[test]
    fn test_add_field_success() {
        let mut proto = Protocol::test_protocol();
        let field1 = FieldRule::new("field1", FieldType::Fixed(0), FieldLength::Fixed(8));

        assert!(proto.add_field(field1).is_ok());
        assert_eq!(proto.fields.len(), 1);
    }

    #[test]
    fn test_add_field_duplicate_id() {
        let mut proto = Protocol::test_protocol();

        proto.with_f("field1", 8);
        assert_eq!(proto.fields.len(), 1);

        let field2 = FieldRule::new("field1", FieldType::Fixed(0), FieldLength::Fixed(16)); // duplicate ID
        assert!(proto.add_field(field2).is_err());
        assert_eq!(proto.fields.len(), 1); // only the first field should be added
    }

    #[test]
    fn test_add_field_after_variable_length_field() {
        let mut proto = Protocol::test_protocol();
        let field1 = FieldRule::new("field1", FieldType::Input, FieldLength::Variable); // field with variable length 
        let field2 = FieldRule::new("field2", FieldType::Fixed(0), FieldLength::Fixed(16)); // field to add after variable length field

        assert!(proto.add_field(field1).is_ok());
        assert_eq!(proto.fields.len(), 1);
        assert!(proto.add_field(field2).is_err());
        assert_eq!(proto.fields.len(), 1);
    }

    #[test]
    fn test_remove_field_success() {
        let mut proto = Protocol::test_protocol();
        proto.with_f("field1", 8);

        assert!(proto.remove_field("field1").is_ok());
        assert_eq!(proto.fields.len(), 0);
    }

    #[test]
    fn test_remove_field_not_found() {
        let mut proto = Protocol::test_protocol();
        proto.with_f("field1", 8);

        assert!(proto.remove_field("nonexistent_field").is_err());
        assert_eq!(proto.fields.len(), 1); // field count should remain unchanged
    }

    #[test]
    fn test_move_field_success() {
        let mut proto = Protocol::test_protocol();
        proto
            .with_f("field1", 8)
            .with_f("field2", 16)
            .with_f("field3", 32);

        proto.move_field("field3", 0).unwrap();
        assert_eq!(proto.fields[0].id, "field3");
        assert_eq!(proto.fields[1].id, "field1");
        assert_eq!(proto.fields[2].id, "field2");
    }

    #[test]
    fn test_move_field_not_found() {
        let mut proto = Protocol::test_protocol();
        proto
            .with_f("field1", 8)
            .with_f("field2", 16)
            .with_f("field3", 32);

        assert!(proto.move_field("nonexistent_field", 1).is_err());
    }

    #[test]
    fn test_move_field_out_of_bounds() {
        let mut proto = Protocol::test_protocol();
        proto
            .with_f("field1", 8)
            .with_f("field2", 16)
            .with_f("field3", 32);

        // Moving to an out-of-bounds index should place the field at the end
        proto.move_field("field1", 10).unwrap();
        assert_eq!(proto.fields[0].id, "field2");
        assert_eq!(proto.fields[1].id, "field3");
        assert_eq!(proto.fields[2].id, "field1");
    }

    #[test]
    fn test_update_field_id_success() {
        let mut proto = Protocol::test_protocol();
        proto.with_f("field1", 8);

        assert!(proto.update_field_id("field1", "field2").is_ok());
        assert_eq!(proto.fields[0].id, "field2");
    }

    #[test]
    fn test_update_field_id_duplicate() {
        let mut proto = Protocol::test_protocol();
        proto.with_f("field1", 8).with_f("field2", 16);

        assert!(proto.update_field_id("field1", "field2").is_err());
        assert_eq!(proto.fields[0].id, "field1"); // ID should remain unchanged
    }

    #[test]
    fn test_edit_field_success() {
        let mut proto = Protocol::test_protocol();
        proto.with_f("field1", 8);

        let result = proto.edit_field("field1", |f| {
            f.length = FieldLength::Fixed(16);
            Ok(())
        });

        assert!(result.is_ok());
        assert_eq!(proto.fields[0].length, FieldLength::Fixed(16));
    }

    #[test]
    fn test_edit_field_id_change_attempt() {
        let mut proto = Protocol::test_protocol();
        proto.with_f("field1", 8);

        let result = proto.edit_field("field1", |f| {
            f.id = "new_field_id".to_string(); // attempt to change ID
            f.length = FieldLength::Fixed(16);
            Ok(())
        });

        assert!(result.is_err());
        // all changes should be reverted
        assert_eq!(proto.fields[0].id, "field1");
        assert!(proto.fields[0].length == FieldLength::Fixed(8));
    }

    #[test]
    fn test_protocol_length_calculation() {
        let mut proto = Protocol::test_protocol();
        proto
            .with_f("field1", 8)
            .with_f("field2", 12)
            .with_f("field3", 4);

        assert_eq!(proto.length, ProtocolLength::Fixed(24));

        // Add a variable length field
        let var_field = FieldRule::new("field4", FieldType::Input, FieldLength::Variable);
        proto.add_field(var_field).unwrap();

        assert_eq!(proto.length, ProtocolLength::Variable(24));
    }

    #[test]
    fn test_empty_protocol_length() {
        let proto = Protocol::test_protocol();
        assert_eq!(proto.length, ProtocolLength::Fixed(0));
    }

    #[test]
    fn test_create_protocol_duplicate_id() {
        let mut registry = ProtocolRegistry::new();
        assert!(
            registry
                .create_protocol("proto1", None, Endianness::Big, None)
                .is_ok()
        );
        assert!(
            registry
                .create_protocol("proto1", None, Endianness::Little, None)
                .is_err()
        );
    }

    #[test]
    fn test_get_protocol_not_found() {
        let registry = ProtocolRegistry::new();
        assert!(registry.get_protocol("nonexistent_proto").is_none());
    }

    #[test]
    fn test_remove_protocol_with_subprotocols() {
        let mut registry = ProtocolRegistry::new();
        registry
            .with_proto("parent_proto", None)
            .with_proto("child_proto", Some("parent_proto".to_string()));

        assert_eq!(registry.protocols.len(), 2);
        assert!(registry.remove_protocol("parent_proto").is_ok());
        assert_eq!(registry.protocols.len(), 0); // both parent and child should be removed
    }

    #[test]
    fn test_update_protocol_id_with_children() {
        let mut registry = ProtocolRegistry::new();
        registry
            .with_proto("parent_proto", None)
            .with_proto("child_proto", Some("parent_proto".to_string()));

        assert!(registry
            .update_protocol_id("parent_proto", "new_parent_proto")
            .is_ok());
        assert!(registry.get_protocol("parent_proto").is_none());
        assert!(registry.get_protocol("new_parent_proto").is_some());

        // Check that the child protocol's parent_id has been updated
        let child_proto = registry.get_protocol("child_proto").unwrap();
        assert_eq!(
            child_proto.parent_id.as_deref(),
            Some("new_parent_proto")
        );
    }

    #[test]
    fn test_edit_protocol_success() {
        let mut registry = ProtocolRegistry::new();
        registry
            .with_proto("proto1", None);

        let result = registry.edit_protocol("proto1", |p| {
            p.name = Some("Data Message".to_string());
            Ok(())
        });

        assert!(result.is_ok());
        let proto1 = registry.get_protocol("proto1").unwrap();
        assert_eq!(proto1.name.as_deref(), Some("Data Message"));
    }

    #[test]
    fn test_edit_protocol_fail() {
        let mut registry = ProtocolRegistry::new();
        registry
            .with_proto("proto1", None);

        registry.edit_protocol("proto1", |p| {
            p.name = Some("Some Name".to_string());
            Ok(())
        }).unwrap();

        let result = registry.edit_protocol("proto1", |p| {
            p.name = Some("Another Name".to_string());
            Err("Failed to edit protocol".to_string())
        });

        assert!(result.is_err());
        let proto1 = registry.get_protocol("proto1").unwrap();
        assert_eq!(proto1.name.as_deref(), Some("Some Name"));
    }

    #[test]
    fn test_attempt_change_parent_id() {
        let mut registry = ProtocolRegistry::new();
        registry
            .with_proto("proto1", None)
            .with_proto("proto2", None);

        let result = registry.edit_protocol("proto2", |p| {
            p.parent_id = Some("proto1".to_string()); // attempt to change parent_id
            Ok(())
        });

        assert!(result.is_err());
        let proto2 = registry.get_protocol("proto2").unwrap();
        assert_eq!(proto2.parent_id, None);
    }

    #[test]
    fn test_get_inheritance_chain() {
        let mut registry = ProtocolRegistry::new();
        registry
            .with_proto("grandparent", None)
            .with_proto("parent", Some("grandparent".to_string()))
            .with_proto("child", Some("parent".to_string()));

        let chain = registry.get_inheritance_chain("child");
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].id, "grandparent");
        assert_eq!(chain[1].id, "parent");
        assert_eq!(chain[2].id, "child");
    }

    #[test]
    fn test_get_total_length() {
        let mut registry = ProtocolRegistry::new();
        registry
            .with_proto("parent", None)
            .with_proto("child", Some("parent".to_string()));

        registry.protocols.get_mut("parent").unwrap()
            .with_f("field1", 8)
            .with_f("field2", 4);
        registry.protocols.get_mut("child").unwrap()
            .with_f("field3", 16);

        let total_length = registry.get_total_length("child");
        assert_eq!(total_length, ProtocolLength::Fixed(28));
    }

    impl Protocol {
        fn test_protocol() -> Self {
            Protocol::new("test_proto", None, Endianness::Big, None)
        }

        fn with_f(&mut self, field_id: &str, field_len: u32) -> &mut Self {
            let field =
                FieldRule::new(field_id, FieldType::Fixed(0), FieldLength::Fixed(field_len));
            self.add_field(field).unwrap();
            self
        }
    }

    impl ProtocolRegistry {
        fn with_proto(&mut self, id: &str, parent_id: Option<String>) -> &mut Self {
            self.create_protocol(id, None, Endianness::Big, parent_id)
                .unwrap();
            self
        }
    }
}
