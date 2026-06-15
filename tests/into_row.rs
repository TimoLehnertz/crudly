use common::mock_db::{MockArguments, MockDB};
use crudly::IntoRow;
use serde::Serialize;
use sqlx::types::Json;

mod common;

fn bind_mock<R: IntoRow<MockDB>>(row: R, args: &mut MockArguments) -> sqlx::Result<()> {
    IntoRow::<MockDB>::bind_arguments(row, args)
}

#[test]
fn rename_all_camel_columns_and_bind_count() {
    #[derive(IntoRow)]
    #[sqlx(rename_all = "camelCase")]
    struct RenameAllCamel {
        user_name: String,
    }

    assert_eq!(
        <RenameAllCamel as IntoRow<MockDB>>::columns(),
        vec!["userName"]
    );
}

#[test]
fn field_rename_maps_column() {
    #[derive(IntoRow)]
    struct RenamedField {
        #[crudly(rename = "street_address")]
        line1: String,
    }
    assert_eq!(
        <RenamedField as IntoRow<MockDB>>::columns(),
        vec!["street_address"]
    );
}

#[test]
fn skip_omits_column_and_bind() {
    #[derive(IntoRow)]
    struct Skipped {
        keep: String,
        #[sqlx(skip)]
        _skip_me: String,
    }
    assert_eq!(<Skipped as IntoRow<MockDB>>::columns(), vec!["keep"]);

    let mut args = MockArguments::default();
    bind_mock(
        Skipped {
            keep: "kept".into(),
            _skip_me: "skip-me".into(),
        },
        &mut args,
    )
    .unwrap();
    assert_eq!(args.values, vec![Some("kept".into())]);
}

#[test]
fn flatten_inlines_inner_columns_and_binds() {
    #[derive(IntoRow)]
    struct InnerFlat {
        b: String,
    }

    #[derive(IntoRow)]
    struct OuterFlat {
        a: String,
        #[sqlx(flatten)]
        inner: InnerFlat,
        c: String,
    }

    assert_eq!(
        <OuterFlat as IntoRow<MockDB>>::columns(),
        vec!["a", "b", "c"]
    );
    let mut args = MockArguments::default();

    bind_mock(
        OuterFlat {
            a: "a".into(),
            inner: InnerFlat { b: "b".into() },
            c: "c".into(),
        },
        &mut args,
    )
    .unwrap();

    assert_eq!(
        args.values,
        vec![Some("a".into()), Some("b".into()), Some("c".into())]
    );
}

#[test]
fn try_from_conversions() {
    #[derive(IntoRow)]
    struct TryIntoField {
        #[crudly(try_into = "String")]
        n: u16,
    }
    let mut args = MockArguments::default();
    bind_mock(TryIntoField { n: 3 }, &mut args).unwrap();
    assert_eq!(args.values, vec![Some("3".into())]);

    #[derive(IntoRow)]
    struct TryFromField {
        #[sqlx(try_from = "String")]
        n: u16,
    }
    let mut args = MockArguments::default();
    bind_mock(TryFromField { n: 3 }, &mut args).unwrap();
    assert_eq!(args.values, vec![Some("3".into())]);
}

/// container default (accepted by the derive; no runtime effect on bind yet)
#[test]
fn container_default_attribute_allowed() {
    #[derive(IntoRow)]
    #[sqlx(default)]
    struct ContainerDefault {
        x: String,
    }
    assert_eq!(<ContainerDefault as IntoRow<MockDB>>::columns(), vec!["x"]);
    let mut args = MockArguments::default();
    bind_mock(ContainerDefault { x: "ok".into() }, &mut args).unwrap();
    assert_eq!(args.values, vec![Some("ok".into())]);
}

/// split keys: sqlx rename_all only
#[test]
fn sqlx_rename_all_only() {
    #[derive(IntoRow)]
    #[sqlx(rename_all = "UPPERCASE")]
    struct SplitAttrs {
        name: String,
    }

    assert_eq!(<SplitAttrs as IntoRow<MockDB>>::columns(), vec!["NAME"]);
}

#[derive(Serialize, Clone)]
pub struct JsonObject {
    pub name: String,
}

/// json (`Json<T>` is recorded as JSON text in the mock)
#[test]
fn json_field_binds() {
    #[derive(IntoRow)]
    struct JsonField {
        #[sqlx(json)]
        json: JsonObject,
    }

    assert_eq!(<JsonField as IntoRow<MockDB>>::columns(), vec!["json"]);
    let json = JsonObject {
        name: "test".into(),
    };

    let mut args = MockArguments::default();
    let expected = serde_json::to_string(&json).unwrap();
    bind_mock(JsonField { json }, &mut args).unwrap();
    assert_eq!(args.values, vec![Some(expected)]);
}

#[derive(IntoRow)]
struct JsonNullable {
    #[crudly(json(nullable))]
    json: Option<JsonObject>,
}

#[test]
fn json_nullable_none() {
    assert_eq!(<JsonNullable as IntoRow<MockDB>>::columns(), vec!["json"]);
    let mut args = MockArguments::default();
    bind_mock(JsonNullable { json: None }, &mut args).unwrap();
    assert_eq!(args.values, vec![None]);
}

#[test]
fn json_nullable_some() {
    let mut args = MockArguments::default();
    let json = JsonObject {
        name: "hi".to_string(),
    };
    bind_mock(
        JsonNullable {
            json: Some(json.clone()),
        },
        &mut args,
    )
    .unwrap();
    let expected = serde_json::to_string(&Json(Some(json))).unwrap();
    assert_eq!(args.values, vec![Some(expected)]);
}

/// field default (attribute accepted; bind uses the field value as usual)
#[test]
fn field_default_attribute_allowed() {
    #[derive(IntoRow)]
    struct FieldDefaultAttr {
        #[sqlx(default)]
        flag: String,
    }

    assert_eq!(
        <FieldDefaultAttr as IntoRow<MockDB>>::columns(),
        vec!["flag"]
    );
    let mut args = MockArguments::default();
    bind_mock(FieldDefaultAttr { flag: "set".into() }, &mut args).unwrap();
    assert_eq!(args.values, vec![Some("set".into())]);
}
