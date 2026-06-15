//! Tests for `#[derive(Schema)]` and default blanket CRUD impls.
#![allow(dead_code, unused_variables)]

use common::sqlite;
use crudly::{
    CrudlyDefault, HasColumns, HasId, Insert, InsertManyWithoutIds, InsertWithoutId, IntoRow,
    Schema, SelectAll,
};
use sqlx::FromRow;

mod common;

#[test]
fn schema_table_inferred_plural_snake() {
    #[derive(FromRow, IntoRow, Schema)]
    struct SimpleRecord {
        #[crudly(id)]
        id: i64,
        name: String,
    }
    impl CrudlyDefault for SimpleRecord {}

    assert_eq!(<SimpleRecord as Schema>::table_name(), "simple_records");
    assert_eq!(<SimpleRecord as HasColumns>::columns(), vec!["name"]);
}

#[test]
fn schema_table_explicit() {
    #[derive(FromRow, IntoRow, Schema)]
    #[crudly(table = "widgets")]
    struct RenamedTable {
        #[crudly(id)]
        id: i64,
        flag: bool,
    }
    impl CrudlyDefault for RenamedTable {}

    assert_eq!(<RenamedTable as Schema>::table_name(), "widgets");
}

#[test]
fn schema_id_column_custom() {
    #[derive(FromRow, IntoRow, Schema)]
    struct CustomKeyColumn {
        #[crudly(id)]
        row_id: i32,
        n: u8,
    }
    impl CrudlyDefault for CustomKeyColumn {}

    assert_eq!(<CustomKeyColumn as HasId>::id_column(), "row_id");
    let row = CustomKeyColumn { row_id: 7, n: 1 };
    assert_eq!(<CustomKeyColumn as HasId>::id(&row), 7);
}

#[tokio::test]
async fn blanket_default_crud_is_available() {
    let pool = sqlite::memory_with_schema(
        r#"CREATE TABLE tiny_rows (tiny_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, label TEXT NOT NULL);"#,
    )
    .await;

    #[derive(Debug, Default, Clone, FromRow, IntoRow, Schema)]
    #[crudly(table = "tiny_rows")]
    struct TinyRow {
        #[crudly(id)]
        #[sqlx(rename = "tiny_id")]
        id: i64,
        label: String,
    }
    impl CrudlyDefault for TinyRow {}

    let all = TinyRow::select_all(&pool).await.unwrap();
    assert!(all.is_empty());

    let id = TinyRow {
        id: 0,
        label: "a".into(),
    }
    .insert(&pool)
    .await
    .unwrap();
    assert_eq!(id, 1);

    TinyRow::insert_many(
        vec![
            TinyRow {
                id: 0,
                label: "b".into(),
            },
            TinyRow {
                id: 0,
                label: "c".into(),
            },
        ],
        1,
        &pool,
    )
    .await
    .unwrap();

    assert_eq!(TinyRow::select_all(&pool).await.unwrap().len(), 3);
}

#[derive(FromRow, IntoRow, Schema)]
struct InventoryCategory {
    #[crudly(id)]
    id: i32,
    title: String,
}
impl CrudlyDefault for InventoryCategory {}

#[test]
fn plural_y_after_consonant() {
    assert_eq!(
        <InventoryCategory as Schema>::table_name(),
        "inventory_categories"
    );
}

#[derive(FromRow, IntoRow, Schema)]
#[crudly(external_ids)]
struct PresetIdRow {
    #[crudly(id)]
    id: i64,
    label: String,
}
impl CrudlyDefault for PresetIdRow {}

#[tokio::test]
async fn external_ids_insert_with_id() {
    let pool = sqlite::memory_with_schema(
        r#"CREATE TABLE preset_id_rows (id INTEGER PRIMARY KEY NOT NULL, label TEXT NOT NULL);"#,
    )
    .await;

    PresetIdRow {
        id: 42,
        label: "hi".into(),
    }
    .insert(&pool)
    .await
    .unwrap();

    let n: i64 = sqlx::query_scalar(r#"SELECT id FROM preset_id_rows WHERE id = 42"#)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 42);
}
