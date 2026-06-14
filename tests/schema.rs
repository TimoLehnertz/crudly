//! Tests deriving Schema instead of Crudly
use crudly::{HasId, IntoRow, Schema};
use sqlx::FromRow;

#[derive(Default, FromRow, IntoRow, Schema)]
#[crudly(table = "usersABC")]
pub struct User {
    #[crudly(id)]
    pub id: i64,
    pub name: String,
}

#[test]
fn test_schema_derive() {
    let user = User {
        id: 1,
        name: "John Doe".to_string(),
    };
    assert_eq!(1, <User as HasId>::id(&user));
    assert_eq!("usersABC", <User as Schema>::table_name());
    assert_eq!("id", <User as HasId>::id_column());
}
