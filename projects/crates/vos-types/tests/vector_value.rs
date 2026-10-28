//! The `vector<N>` value contract: fixed dimension, `f32` precision, finite
//! components and serialization order.

use vos_types::{VECTOR_VALUE_FORMAT_VERSION, VectorElement, VectorValue, VectorValueWire};

fn wire(dimension: u32, components: Vec<f32>) -> VectorValueWire {
    VectorValueWire {
        format_version: VECTOR_VALUE_FORMAT_VERSION.to_owned(),
        element: "f32".to_owned(),
        dimension,
        components,
    }
}

#[test]
fn dimension_precision_and_component_order_survive_a_round_trip() {
    let value = VectorValue::new(3, vec![3.0, 1.0, 2.0]).unwrap();
    assert_eq!(value.dimension(), 3);
    assert_eq!(value.element(), VectorElement::F32);
    assert_eq!(
        value.components(),
        &[3.0, 1.0, 2.0],
        "components keep their index order and are never sorted"
    );

    let json = value.to_json().unwrap();
    assert_eq!(VectorValue::from_json(&json).unwrap(), value);
    assert_eq!(
        value.to_wire(),
        wire(3, vec![3.0, 1.0, 2.0]),
        "the wire carries the precision and the dimension explicitly"
    );
}

#[test]
fn f32_precision_is_preserved_exactly() {
    let value = VectorValue::new(2, vec![0.1, -0.25]).unwrap();
    let decoded = VectorValue::from_json(&value.to_json().unwrap()).unwrap();
    assert_eq!(decoded.components(), value.components());
    assert_eq!(decoded.components()[0], 0.1_f32);
}

#[test]
fn zero_dimension_is_rejected() {
    let error = VectorValue::new(0, Vec::new()).unwrap_err();
    assert_eq!(error.code, "VEC001");
}

#[test]
fn component_count_must_match_the_dimension() {
    let error = VectorValue::new(3, vec![1.0, 2.0]).unwrap_err();
    assert_eq!(error.code, "VEC002");
    assert_eq!(
        error.message,
        "vector of dimension 3 must carry 3 components"
    );

    let error = VectorValue::new(2, vec![1.0, 2.0, 3.0]).unwrap_err();
    assert_eq!(error.code, "VEC002");
}

#[test]
fn non_finite_components_are_rejected() {
    for component in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let error = VectorValue::new(2, vec![1.0, component]).unwrap_err();
        assert_eq!(error.code, "VEC003");
        assert!(
            error.message.contains("finite"),
            "the diagnostic must name the finite-value rule"
        );
    }
}

#[test]
fn unsupported_element_precision_is_rejected() {
    let mut wire = wire(2, vec![1.0, 2.0]);
    wire.element = "f64".to_owned();
    assert_eq!(VectorValue::from_wire(wire).unwrap_err().code, "VEC004");
    assert_eq!(VectorElement::from_name("f64"), None);
    assert_eq!(VectorElement::from_name("f32"), Some(VectorElement::F32));
    assert_eq!(VectorElement::F32.as_str(), "f32");
}

#[test]
fn stale_format_version_is_rejected() {
    let mut wire = wire(2, vec![1.0, 2.0]);
    wire.format_version = "vos-vector-value-v1".to_owned();
    assert_eq!(VectorValue::from_wire(wire).unwrap_err().code, "VEC005");
}

#[test]
fn unknown_wire_fields_are_rejected() {
    let input = r#"{
      "formatVersion": "vos-vector-value-v0",
      "element": "f32",
      "dimension": 2,
      "components": [1.0, 2.0],
      "metric": "cosine"
    }"#;
    assert_eq!(VectorValue::from_json(input).unwrap_err().code, "VEC006");
}
