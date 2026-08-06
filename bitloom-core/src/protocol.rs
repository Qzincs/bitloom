use super::field::{Field, FieldLength, FieldRule, ResolvedField};
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
    /// Returns all fields with the protocol where each field was defined.
    pub fn resolve_fields(&self, protocol_id: &str) -> Result<Vec<ResolvedField>, String> {
        let chain = self.get_inheritance_chain(protocol_id);
        if chain.is_empty() {
            return Err(format!("Protocol with ID '{}' does not exist", protocol_id));
        }

        let total_fields: usize = chain.iter().map(|p| p.fields.len()).sum();
        let mut resolved_fields = Vec::with_capacity(total_fields);

        for proto in chain {
            resolved_fields.extend(proto.fields.iter().map(|field| ResolvedField {
                field: field.clone(),
                defined_in: proto.id.clone(),
            }));
        }

        Ok(resolved_fields)
    }

    /// Build a tree structure of protocols for UI display, where each protocol is represented as a node and its subprotocols are its children.
    pub fn build_protocol_trees(&self) -> Vec<ProtocolTreeNode> {
        use std::collections::HashMap;

        // Group protocols by their parent_id so we avoid repeated lookups when building the tree.
        let mut parent_to_children: HashMap<Option<String>, Vec<String>> = HashMap::new();
        for proto in self.protocols.values() {
            parent_to_children
                .entry(proto.parent_id.clone())
                .or_default()
                .push(proto.id.clone());
        }

        // sort children lists for consistent ordering in the UI
        for children in parent_to_children.values_mut() {
            children.sort();
        }

        // build tree nodes recursively
        fn build_recursive(
            current_id: &str,
            map: &HashMap<Option<String>, Vec<String>>,
            registry: &ProtocolRegistry,
        ) -> ProtocolTreeNode {
            let protocol = registry.protocols.get(current_id);
            let mut node = ProtocolTreeNode {
                id: current_id.to_string(),
                remark: protocol.and_then(|p| p.remark.clone()),
                children: Vec::new(),
            };

            if let Some(child_ids) = map.get(&Some(current_id.to_string())) {
                for child_id in child_ids {
                    node.children.push(build_recursive(child_id, map, registry));
                }
            }
            node
        }

        // build trees starting from root protocols
        parent_to_children
            .get(&None) // all root protocols (those without a parent)
            .cloned()
            .unwrap_or_default() // in case there are no root protocols
            .into_iter()
            .map(|root_id| build_recursive(&root_id, &parent_to_children, self)) // build each tree
            .collect()
    }

    pub fn add_protocol_field(
        &mut self,
        protocol_id: &str,
        field_rule: FieldRule,
    ) -> Result<(), String> {
        if let Some(proto) = self.protocols.get_mut(protocol_id) {
            proto.add_field(field_rule)
        } else {
            Err(format!("Protocol with ID '{}' does not exist", protocol_id))
        }
    }

    pub fn remove_protocol_field(
        &mut self,
        protocol_id: &str,
        field_id: &str,
    ) -> Result<(), String> {
        if let Some(proto) = self.protocols.get_mut(protocol_id) {
            proto.remove_field(field_id)
        } else {
            Err(format!("Protocol with ID '{}' does not exist", protocol_id))
        }
    }

    pub fn move_protocol_field(
        &mut self,
        protocol_id: &str,
        field_id: &str,
        new_index: usize,
    ) -> Result<(), String> {
        if let Some(proto) = self.protocols.get_mut(protocol_id) {
            proto.move_field(field_id, new_index)
        } else {
            Err(format!("Protocol with ID '{}' does not exist", protocol_id))
        }
    }

    pub fn update_protocol_field_id(
        &mut self,
        protocol_id: &str,
        old_field_id: &str,
        new_field_id: &str,
    ) -> Result<(), String> {
        if let Some(proto) = self.protocols.get_mut(protocol_id) {
            proto.update_field_id(old_field_id, new_field_id)
        } else {
            Err(format!("Protocol with ID '{}' does not exist", protocol_id))
        }
    }

    pub fn edit_protocol_field<F>(
        &mut self,
        protocol_id: &str,
        field_id: &str,
        f: F,
    ) -> Result<(), String>
    where
        F: FnOnce(&mut FieldRule) -> Result<(), String>,
    {
        if let Some(proto) = self.protocols.get_mut(protocol_id) {
            proto.edit_field(field_id, f)
        } else {
            Err(format!("Protocol with ID '{}' does not exist", protocol_id))
        }
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
