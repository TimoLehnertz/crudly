//! An entity may have no column besides its id, and a flattened `()` adds none.

use common::sqlite;
use crudly::{
    CrudlyDefault, HasColumns, InsertManyWithoutIds, InsertWithoutId, IntoRow, Schema, SelectAll,
    UpdateById,
};
use sqlx::FromRow;

mod common;

#[derive(Debug, PartialEq, FromRow, IntoRow, Schema)]
#[crudly(table = "tickets")]
struct Ticket {
    #[crudly(id)]
    id: i64,
}
impl CrudlyDefault for Ticket {}

#[derive(IntoRow)]
struct Stamp<Extra> {
    ink: String,
    #[crudly(flatten)]
    extra: Extra,
}

#[test]
fn a_flattened_unit_adds_no_column() {
    assert_eq!(<Stamp<()> as HasColumns>::columns(), vec!["ink"]);
    assert!(<Ticket as HasColumns>::columns().is_empty());
}

#[tokio::test]
async fn an_entity_with_only_an_id_is_inserted_and_updated() {
    let pool =
        sqlite::memory_with_schema("CREATE TABLE tickets (id INTEGER PRIMARY KEY NOT NULL);").await;

    let first = InsertWithoutId::insert(Ticket { id: 0 }, &pool)
        .await
        .unwrap();
    InsertManyWithoutIds::insert_many(vec![Ticket { id: 0 }, Ticket { id: 0 }], 0, &pool)
        .await
        .unwrap();
    assert_eq!(Ticket::select_all(&pool).await.unwrap().len(), 3);

    // Nothing to set: the update only says whether the row exists.
    assert!(Ticket { id: first }.update_by_id(&pool).await.unwrap());
    assert!(!Ticket { id: 99 }.update_by_id(&pool).await.unwrap());
}
