use super::super::field::FieldKind;
use super::*;

#[test]
fn test_add_field_success() {
    let mut proto = Protocol::test_protocol();
    let field1 = FieldRule::new("field1", FieldKind::Fixed(0), FieldLength::Fixed(8));

    assert!(proto.add_field(field1).is_ok());
    assert_eq!(proto.fields.len(), 1);
}

#[test]
fn test_add_field_duplicate_id() {
    let mut proto = Protocol::test_protocol();

    proto.with_f("field1", 8);
    assert_eq!(proto.fields.len(), 1);

    let field2 = FieldRule::new("field1", FieldKind::Fixed(0), FieldLength::Fixed(16)); // duplicate ID
    assert!(proto.add_field(field2).is_err());
    assert_eq!(proto.fields.len(), 1); // only the first field should be added
}

#[test]
fn test_add_field_after_variable_length_field() {
    let mut proto = Protocol::test_protocol();
    let field1 = FieldRule::new("field1", FieldKind::Input, FieldLength::Variable); // field with variable length
    let field2 = FieldRule::new("field2", FieldKind::Fixed(0), FieldLength::Fixed(16)); // field to add after variable length field

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
fn test_move_variable_field_before_fixed_field() {
    let mut proto = Protocol::test_protocol();
    proto.with_f("field1", 8);
    proto
        .add_field(FieldRule::new(
            "field2",
            FieldKind::Input,
            FieldLength::Variable,
        ))
        .unwrap();

    assert!(proto.move_field("field2", 0).is_err());
    assert_eq!(proto.fields[0].id, "field1");
    assert_eq!(proto.fields[1].id, "field2");
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
fn test_edit_non_last_field_to_variable_length() {
    let mut proto = Protocol::test_protocol();
    proto.with_f("field1", 8).with_f("field2", 16);

    let result = proto.edit_field("field1", |field| {
        field.length = FieldLength::Variable;
        Ok(())
    });

    assert!(result.is_err());
    assert_eq!(proto.fields[0].length, FieldLength::Fixed(8));
    assert_eq!(proto.fields[1].length, FieldLength::Fixed(16));
    assert_eq!(proto.length, ProtocolLength::Fixed(24));
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
    let var_field = FieldRule::new("field4", FieldKind::Input, FieldLength::Variable);
    proto.add_field(var_field).unwrap();

    assert_eq!(proto.length, ProtocolLength::Variable(24));
}

#[test]
fn test_empty_protocol_length() {
    let proto = Protocol::test_protocol();
    assert_eq!(proto.length, ProtocolLength::Fixed(0));
}

impl Protocol {
    fn test_protocol() -> Self {
        Protocol::new("test_proto", None, Endianness::Big, None)
    }

    fn with_f(&mut self, field_id: &str, field_len: u32) -> &mut Self {
        let field = FieldRule::new(field_id, FieldKind::Fixed(0), FieldLength::Fixed(field_len));
        self.add_field(field).unwrap();
        self
    }
}
