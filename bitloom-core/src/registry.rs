use crate::field::{FieldLength, FieldRule, ResolvedField};
use crate::protocol::{Endianness, Protocol, ProtocolLength, ProtocolTreeNode};
use std::collections::HashMap;

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
        let mut candidate = self
            .protocols
            .get(protocol_id)
            .cloned()
            .ok_or_else(|| format!("Protocol with ID '{}' does not exist", protocol_id))?;
        candidate.add_field(field_rule)?;

        self.validate_protocol(&candidate)?;

        self.protocols.insert(protocol_id.to_string(), candidate);
        Ok(())
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
        let mut candidate = self
            .protocols
            .get(protocol_id)
            .cloned()
            .ok_or_else(|| format!("Protocol with ID '{}' does not exist", protocol_id))?;
        candidate.move_field(field_id, new_index)?;

        self.validate_protocol(&candidate)?;

        self.protocols.insert(protocol_id.to_string(), candidate);
        Ok(())
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
        let mut candidate = self
            .protocols
            .get(protocol_id)
            .cloned()
            .ok_or_else(|| format!("Protocol with ID '{}' does not exist", protocol_id))?;
        candidate.edit_field(field_id, f)?;

        self.validate_protocol(&candidate)?;

        self.protocols.insert(protocol_id.to_string(), candidate);
        Ok(())
    }

    fn validate_protocol(&self, protocol: &Protocol) -> Result<(), String> {
        let chain = self.get_inheritance_chain(&protocol.id);
        let mut all_fields = Vec::new();

        for i in 0..chain.len() - 1 {
            all_fields.extend(chain[i].fields.iter().cloned());
        }

        all_fields.extend(protocol.fields.iter().cloned());

        Protocol::validate_field_layout(&all_fields, &protocol.id)?;

        let Some(last_field) = all_fields.last() else {
            return Ok(());
        };

        if matches!(last_field.length, FieldLength::Variable) {
            if self.has_descendant_fields(&protocol.id) {
                return Err(format!(
                    "Protocol '{}' has descendants with fields, so its last field cannot be variable",
                    protocol.id
                ));
            }
        }

        Ok(())
    }

    fn has_descendant_fields(&self, protocol_id: &str) -> bool {
        let children = self
            .protocols
            .values()
            .filter(|p| p.parent_id.as_deref() == Some(protocol_id));

        for child in children {
            if !child.fields.is_empty() || self.has_descendant_fields(&child.id) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
