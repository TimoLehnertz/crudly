//! `#[derive(IntoRow)]` coverage using [`MockDB`](common::mock_db::MockDB) and [`MockArguments`].
//!
//! Non-string payloads use `try_into = "String"` (implemented as [`ToString`] then bind) or
//! `sqlx::types::Json`; the mock records each bound value as text.

use crudly::{HasColumns, IntoRow};
use serde::Serialize;
use sqlx::Arguments;
use sqlx::types::Json;

mod common;

use common::mock_db::{MockArguments, MockDB};

fn bind_mock<R: IntoRow<MockDB>>(row: R, args: &mut MockArguments<'_>) -> sqlx::Result<()> {
    IntoRow::<MockDB>::bind_arguments(row, args)
}

// --- rename_all

#[derive(IntoRow)]
#[sqlx(rename_all = "camelCase")]
struct RenameAllCamel {
    user_name: String,
}

#[test]
fn rename_all_camel_columns_and_bind_count() {
    assert_eq!(RenameAllCamel::columns(), vec!["userName"]);
    let row = RenameAllCamel {
        user_name: "n".into(),
    };
    let mut args = MockArguments::default();
    bind_mock(row, &mut args).unwrap();
    assert_eq!(args.len(), 1);
    assert_eq!(args.values, vec![Some("n".into())]);
}

// --- field rename

#[derive(IntoRow)]
struct RenamedField {
    #[crudly(rename = "street_address")]
    line1: String,
}

#[test]
fn field_rename_maps_column() {
    assert_eq!(RenamedField::columns(), vec!["street_address"]);
    let mut args = MockArguments::default();
    bind_mock(
        RenamedField {
            line1: "1 ave".into(),
        },
        &mut args,
    )
    .unwrap();
    assert_eq!(args.len(), 1);
    assert_eq!(args.values, vec![Some("1 ave".into())]);
}

// --- skip

#[derive(IntoRow)]
struct Skipped {
    keep: String,
    #[sqlx(skip)]
    _noise: String,
}

#[test]
fn skip_omits_column_and_bind() {
    assert_eq!(Skipped::columns(), vec!["keep"]);
    let mut args = MockArguments::default();
    bind_mock(
        Skipped {
            keep: "kept".into(),
            _noise: "skip-me".into(),
        },
        &mut args,
    )
    .unwrap();
    assert_eq!(args.len(), 1);
    assert_eq!(args.values, vec![Some("kept".into())]);
}

// --- flatten

#[derive(IntoRow)]
struct InnerFlat {
    a: String,
}

#[derive(IntoRow)]
struct OuterFlat {
    #[sqlx(flatten)]
    inner: InnerFlat,
    b: String,
}

#[test]
fn flatten_inlines_inner_columns_and_binds() {
    assert_eq!(OuterFlat::columns(), vec!["a", "b"]);
    let mut args = MockArguments::default();
    bind_mock(
        OuterFlat {
            inner: InnerFlat { a: "inner".into() },
            b: "outer".into(),
        },
        &mut args,
    )
    .unwrap();
    assert_eq!(args.len(), 2);
    assert_eq!(
        args.values,
        vec![Some("inner".into()), Some("outer".into())]
    );
}

// --- try_from / try_into (field → bind `field.to_string()` when target type is `String`)

#[derive(IntoRow)]
struct TryNarrow {
    #[sqlx(try_into = "String")]
    n: u16,
}

#[test]
fn try_into_converts_before_bind() {
    assert_eq!(TryNarrow::columns(), vec!["n"]);
    let mut args = MockArguments::default();
    bind_mock(TryNarrow { n: 3 }, &mut args).unwrap();
    assert_eq!(args.values, vec![Some("3".into())]);
}

#[derive(IntoRow)]
struct TryFromAttr {
    #[crudly(try_from = "String")]
    n: u16,
}

#[test]
fn try_from_same_as_try_into_for_binding() {
    let mut args = MockArguments::default();
    bind_mock(TryFromAttr { n: 5 }, &mut args).unwrap();
    assert_eq!(args.values, vec![Some("5".into())]);
}

// --- container default (accepted by the derive; no runtime effect on bind yet)

#[derive(IntoRow)]
#[sqlx(default)]
struct ContainerDefault {
    x: String,
}

#[test]
fn container_default_attribute_allowed() {
    assert_eq!(ContainerDefault::columns(), vec!["x"]);
    let mut args = MockArguments::default();
    bind_mock(ContainerDefault { x: "ok".into() }, &mut args).unwrap();
    assert_eq!(args.values, vec![Some("ok".into())]);
}

// --- split keys: sqlx rename_all only

#[derive(IntoRow)]
#[sqlx(rename_all = "UPPERCASE")]
struct SplitAttrs {
    name: String,
}

#[test]
fn sqlx_rename_all_only() {
    assert_eq!(SplitAttrs::columns(), vec!["NAME"]);
    let mut args = MockArguments::default();
    bind_mock(SplitAttrs { name: "Ada".into() }, &mut args).unwrap();
    assert_eq!(args.values, vec![Some("Ada".into())]);
}

// --- json (`Json<T>` is recorded as JSON text in the mock)

#[derive(Serialize, Clone)]
pub struct JsonObject {
    pub name: String,
}

#[derive(IntoRow)]
struct JsonField {
    #[sqlx(json)]
    json: JsonObject,
}

#[test]
fn json_field_binds() {
    assert_eq!(JsonField::columns(), vec!["json"]);
    let json = JsonObject {
        name: "test".into(),
    };

    let expected = serde_json::to_string(&json).unwrap();
    let mut args = MockArguments::default();
    bind_mock(JsonField { json }, &mut args).unwrap();
    assert_eq!(args.len(), 1);
    assert_eq!(args.values, vec![Some(expected)]);
}

#[derive(IntoRow)]
struct JsonNullable {
    #[crudly(json(nullable))]
    extra: Option<JsonObject>,
}

#[test]
fn json_nullable_none() {
    assert_eq!(JsonNullable::columns(), vec!["extra"]);
    let mut args = MockArguments::default();
    bind_mock(JsonNullable { extra: None }, &mut args).unwrap();
    let expected = serde_json::to_string(&Json(None::<String>)).unwrap();
    assert_eq!(args.values, vec![Some(expected)]);
}

#[test]
fn json_nullable_some() {
    let mut args = MockArguments::default();
    let json = JsonObject {
        name: "hi".to_string(),
    };
    bind_mock(
        JsonNullable {
            extra: Some(json.clone()),
        },
        &mut args,
    )
    .unwrap();
    let expected = serde_json::to_string(&Json(Some(json))).unwrap();
    assert_eq!(args.values, vec![Some(expected)]);
}

// --- field default (attribute accepted; bind uses the field value as usual)

#[derive(IntoRow)]
struct FieldDefaultAttr {
    #[sqlx(default)]
    flag: String,
}

#[test]
fn field_default_attribute_allowed() {
    assert_eq!(FieldDefaultAttr::columns(), vec!["flag"]);
    let mut args = MockArguments::default();
    bind_mock(FieldDefaultAttr { flag: "set".into() }, &mut args).unwrap();
    assert_eq!(args.values, vec![Some("set".into())]);
}
