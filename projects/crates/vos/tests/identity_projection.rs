//! Operation and service identity stability across source formatting and units.

use vos::parse_oak;

#[test]
fn operation_identity_ignores_whitespace_newlines_and_comments() {
    let compact = parse_oak("query active(status: utf8, limit: i64) -> [User] { User }")
        .unwrap()
        .project_operations()
        .unwrap();
    let formatted = parse_oak(
        "// leading comment\nquery   active ( status : utf8 ,  limit :i64 )\n  -> [ User ]\n{\n  // body comment\n  User\n}\n",
    )
    .unwrap()
    .project_operations()
    .unwrap();

    assert_eq!(compact.len(), 1);
    assert_eq!(formatted.len(), 1);
    assert_eq!(compact[0].identity, formatted[0].identity);
    assert_eq!(compact[0].operation_id, formatted[0].operation_id);
}

#[test]
fn operation_identity_is_stable_across_repeated_builds() {
    let source = "macro public_name(value: utf8) -> utf8 { value.trim() }";
    let first = parse_oak(source).unwrap().project_operations().unwrap();
    let second = parse_oak(source).unwrap().project_operations().unwrap();

    assert_eq!(first[0].identity, second[0].identity);
    assert_eq!(first[0].operation_id, second[0].operation_id);
}

#[test]
fn operation_identity_is_independent_of_declaration_order() {
    let first = parse_oak(
        "macro public_name(value: utf8) -> utf8 { value.trim() }\n\
         query active(status: utf8) -> [User] { User }",
    )
    .unwrap()
    .project_operations()
    .unwrap();
    let second = parse_oak(
        "query active(status: utf8) -> [User] { User }\n\
         macro public_name(value: utf8) -> utf8 { value.trim() }",
    )
    .unwrap()
    .project_operations()
    .unwrap();

    let by_name = |operations: &[vos::types::OperationProjection]| {
        operations
            .iter()
            .map(|operation| (operation.name.clone(), operation.operation_id.clone()))
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    assert_eq!(by_name(&first), by_name(&second));
}

#[test]
fn renaming_an_operation_changes_its_identity() {
    let original = parse_oak("query active(status: utf8) -> [User] { User }")
        .unwrap()
        .project_operations()
        .unwrap();
    let renamed = parse_oak("query activated(status: utf8) -> [User] { User }")
        .unwrap()
        .project_operations()
        .unwrap();

    assert_ne!(original[0].operation_id, renamed[0].operation_id);
}

#[test]
fn service_identity_ignores_formatting_and_tracks_the_canonical_path() {
    let compact = parse_oak("namespace demo::auth\nservice IdentityService { create() }")
        .unwrap()
        .project_services()
        .unwrap();
    let formatted =
        parse_oak("namespace   demo :: auth\n\n\nservice   IdentityService\n{\n  create ( )\n}\n")
            .unwrap()
            .project_services()
            .unwrap();
    let moved = parse_oak("namespace demo::auth\nservice SessionService { create() }")
        .unwrap()
        .project_services()
        .unwrap();

    assert_eq!(compact[0].service_id, formatted[0].service_id);
    assert_ne!(compact[0].service_id, moved[0].service_id);
}
