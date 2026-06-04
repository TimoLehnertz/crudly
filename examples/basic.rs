#![allow(unused_variables)]
use crudly::{Crudly, InsertWithoutId, IntoRow};
use sqlx::{FromRow, SqlitePool, query};

const CREATE_USERS_TABLE_SQL: &str =
    "CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, name TEXT NOT NULL);";

#[derive(FromRow, IntoRow, Crudly, Default)]
struct User {
    #[crudly(id)]
    pub id: i64,
    pub name: String,
}

#[tokio::main]
async fn main() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    query(CREATE_USERS_TABLE_SQL).execute(&pool).await.unwrap();

    // --------------- Inserting a user ---------------

    let user = User {
        id: 0,
        name: "John Doe".to_string(),
    };
    let inserted_id = user.insert(&pool).await.unwrap();
    assert_eq!(inserted_id, 1);

    // --------------- Selecting all users ---------------

    let mut users: Vec<User> = User::select_all(&pool).await.unwrap();
    assert_eq!(users.len(), 1);

    let mut jon = users.remove(0);

    jon.name = "Jane Doe".to_string();

    // --------------- Updating a user ---------------

    let updated = jon.update_by_id(&pool).await.unwrap();

    assert!(updated);

    // --------------- Getting a user by id ---------------

    let updated_user = User::select_by_id(&1, &pool).await.unwrap().unwrap();
    assert_eq!(updated_user.name, "Jane Doe");

    // --------------- Deleting a user ---------------

    let deleted = User::delete_by_id(&1, &pool).await.unwrap();
    assert!(deleted);

    let users: Vec<User> = User::select_all(&pool).await.unwrap();
    assert!(users.is_empty());
}
