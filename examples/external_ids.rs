use crudly::{CrudlyDefault, Insert, IntoRow, Schema, SelectAll};
use sqlx::{FromRow, SqlitePool, query};

#[derive(FromRow, IntoRow, Schema)]
#[crudly(external_ids)] // Here the id will be assigned inside of rust instead of in the db
struct User {
    #[crudly(id)]
    id: i64,
    name: String,
}

impl CrudlyDefault for User {}

const CREATE_USERS_TABLE_SQL: &str =
    "CREATE TABLE users (id INTEGER PRIMARY KEY NOT NULL, name TEXT NOT NULL);";

#[tokio::main]
async fn main() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    query(CREATE_USERS_TABLE_SQL).execute(&pool).await.unwrap();

    // --------------- Inserting a user ---------------

    let user = User {
        id: 42,
        name: "John Doe".to_string(),
    };
    user.insert(&pool).await.unwrap();

    // --------------- Selecting all users ---------------

    let users: Vec<User> = User::select_all(&pool).await.unwrap();
    assert_eq!(users.len(), 1);

    assert_eq!(users[0].id, 42);
}
