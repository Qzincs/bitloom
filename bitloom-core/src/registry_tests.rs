use crate::field::{FieldKind, FieldLength, FieldRule};
use super::*;

#[test]
fn test_create_protocol_duplicate_id() {
    let mut registry = ProtocolRegistry::new();
    assert!(registry.create_protocol("proto1", None, Endianness::Big, None).is_ok());
    assert!(registry.create_protocol("proto1", None, Endianness::Little, None).is_err());
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
    assert_eq!(registry.protocols.len(), 0);
}

#[test]
fn test_update_protocol_id_with_children() {
    let mut registry = ProtocolRegistry::new();
    registry
        .with_proto("parent_proto", None)
        .with_proto("child_proto", Some("parent_proto".to_string()));

    assert!(registry.update_protocol_id("parent_proto", "new_parent_proto").is_ok());
    assert!(registry.get_protocol("parent_proto").is_none());
    assert!(registry.get_protocol("new_parent_proto").is_some());

    let child_proto = registry.get_protocol("child_proto").unwrap();
    assert_eq!(child_proto.parent_id.as_deref(), Some("new_parent_proto"));
}

#[test]
fn test_edit_protocol_success() {
    let mut registry = ProtocolRegistry::new();
    registry.with_proto("proto1", None);

    let result = registry.edit_protocol("proto1", |p| {
        p.remark = Some("Data Message".to_string());
        Ok(())
    });

    assert!(result.is_ok());
    let proto1 = registry.get_protocol("proto1").unwrap();
    assert_eq!(proto1.remark.as_deref(), Some("Data Message"));
}

#[test]
fn test_edit_protocol_fail() {
    let mut registry = ProtocolRegistry::new();
    registry.with_proto("proto1", None);

    registry
        .edit_protocol("proto1", |p| {
            p.remark = Some("Some Name".to_string());
            Ok(())
        })
        .unwrap();

    let result = registry.edit_protocol("proto1", |p| {
        p.remark = Some("Another Name".to_string());
        Err("Failed to edit protocol".to_string())
    });

    assert!(result.is_err());
    let proto1 = registry.get_protocol("proto1").unwrap();
    assert_eq!(proto1.remark.as_deref(), Some("Some Name"));
}

#[test]
fn test_attempt_change_parent_id() {
    let mut registry = ProtocolRegistry::new();
    registry.with_proto("proto1", None).with_proto("proto2", None);

    let result = registry.edit_protocol("proto2", |p| {
        p.parent_id = Some("proto1".to_string());
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

    registry
        .add_protocol_field("parent", FieldRule::new("field1", FieldKind::Input, FieldLength::Fixed(8)))
        .unwrap();
    registry
        .add_protocol_field("parent", FieldRule::new("field2", FieldKind::Input, FieldLength::Fixed(4)))
        .unwrap();
    registry
        .add_protocol_field("child", FieldRule::new("field3", FieldKind::Input, FieldLength::Fixed(16)))
        .unwrap();

    let total_length = registry.get_total_length("child");
    assert_eq!(total_length, ProtocolLength::Fixed(28));
}

impl ProtocolRegistry {
    fn with_proto(&mut self, id: &str, parent_id: Option<String>) -> &mut Self {
        self.create_protocol(id, None, Endianness::Big, parent_id)
            .unwrap();
        self
    }
}
