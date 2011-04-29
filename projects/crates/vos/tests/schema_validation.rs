//! Facade schema validation goes through Oak and the VOS schema contract.

use vos::validate_schema;

#[test]
fn schema_validation_uses_oak_and_requires_table_primary() {
    assert!(validate_schema("table User { @@id: uuid }").is_ok());
    assert!(validate_schema("table User { id: uuid }").is_err());
    assert!(validate_schema("table User {").is_err());
}
