//! Tests for `#[derive(Crudly)]`.
#![allow(dead_code, unused_variables)]

mod common;

use common::mock_crud_executor::{self as mcx, MockCrudExecutor};
use common::sqlite;
use crudly::{Crudly, HasColumns, InsertWithId, InsertWithoutId, IntoRow, Schema};
use sqlx::FromRow;
use sqlx::sqlite::Sqlite;

#[test]
fn schema_table_inferred_plural_snake() {
    #[derive(FromRow, IntoRow, Crudly)]
    struct SimpleRecord {
        #[crudly(id)]
        id: i64,
        name: String,
    }
    assert_eq!(
        <SimpleRecord as Schema<Sqlite>>::table_name(),
        "simple_records"
    );
    assert_eq!(SimpleRecord::columns(), vec!["name"]);
}

#[test]
fn schema_table_explicit() {
    #[derive(FromRow, IntoRow, Crudly)]
    #[crudly(table = "widgets")]
    struct RenamedTable {
        #[crudly(id)]
        id: i64,
        flag: bool,
    }

    assert_eq!(<RenamedTable as Schema<Sqlite>>::table_name(), "widgets");
}

#[test]
fn schema_id_column_custom() {
    #[derive(FromRow, IntoRow, Crudly)]
    struct CustomKeyColumn {
        #[crudly(id)]
        row_id: i32,
        n: u8,
    }

    assert_eq!(<CustomKeyColumn as Schema<Sqlite>>::id_column(), "row_id");
    let row = CustomKeyColumn { row_id: 7, n: 1 };
    assert_eq!(<CustomKeyColumn as Schema<Sqlite>>::id(&row), 7);
}

#[tokio::test]
async fn custom_executor_routes_through_mock() {
    let pool = sqlite::memory_with_schema(
        r#"CREATE TABLE tiny_rows (tiny_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, label TEXT NOT NULL);"#,
    )
    .await;

    #[derive(Debug, Default, Clone, FromRow, IntoRow, Crudly)]
    #[crudly(table = "tiny_rows", executor = MockCrudExecutor)]
    struct TinyRow {
        #[crudly(id)]
        #[crudly(rename = "tiny_id")]
        id: i64,
        label: String,
    }

    mcx::clear_log();
    TinyRow::select_all(&pool).await.unwrap();
    assert_eq!(mcx::take_log(), vec!["select_all"]);

    mcx::clear_log();
    TinyRow {
        id: 0,
        label: "a".into(),
    }
    .insert(&pool)
    .await
    .unwrap();
    assert!(mcx::take_log().contains(&"insert"));

    mcx::clear_log();
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
    assert!(mcx::take_log().contains(&"insert_many_without_id"));
}

#[derive(FromRow, IntoRow, Crudly)]
struct InventoryCategory {
    #[crudly(id)]
    id: i32,
    title: String,
}

#[test]
fn plural_y_after_consonant() {
    assert_eq!(
        <InventoryCategory as Schema<Sqlite>>::table_name(),
        "inventory_categories"
    );
}

#[derive(FromRow, IntoRow, Crudly)]
#[crudly(external_ids)]
struct PresetIdRow {
    #[crudly(id)]
    id: i64,
    label: String,
}

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
