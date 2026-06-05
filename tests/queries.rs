use crudly::{Crudly, CrudlyDefault, InsertWithId, InsertWithoutId, IntoRow, Schema};
use sqlx::{Sqlite, SqlitePool, prelude::FromRow, query};

#[derive(FromRow, IntoRow, Schema, Default, Clone, PartialEq, Debug)]
#[crudly(table = "users")]
pub struct UserInternalID {
    #[crudly(id)]
    pub id: i64,
    pub name: String,
}
impl CrudlyDefault<Sqlite> for UserInternalID {}

#[derive(FromRow, IntoRow, Schema, Default, Clone, PartialEq, Debug)]
#[crudly(external_ids, table = "users")]
pub struct UserExternalID {
    #[crudly(id)]
    pub id: i64,
    pub name: String,
}
impl CrudlyDefault<Sqlite> for UserExternalID {}

async fn sqlite_mem() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    query("CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL);")
        .execute(&pool)
        .await
        .unwrap();
    pool
}

#[tokio::test]
async fn test_insert_external_id() {
    let pool = sqlite_mem().await;
    let user = UserExternalID {
        id: 42,
        name: "John Doe".to_string(),
    };
    user.insert(&pool).await.unwrap();

    let user = UserExternalID::select_by_id(&42, &pool)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user.id, 42);
}

#[tokio::test]
async fn test_insert_db_id() {
    let pool = sqlite_mem().await;
    let user = UserInternalID {
        id: 42,
        name: "John Doe".to_string(),
    };
    let user_id = user.insert(&pool).await.unwrap();

    let user = UserInternalID::select_by_id(&user_id, &pool)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user.id, user_id);
}

#[tokio::test]
async fn test_insert_many_external_ids() {
    let pool = sqlite_mem().await;
    let users = vec![
        UserExternalID {
            id: 1,
            ..Default::default()
        },
        UserExternalID {
            id: 2,
            ..Default::default()
        },
        UserExternalID {
            id: 3,
            ..Default::default()
        },
    ];
    UserExternalID::insert_many(users.clone(), 2, &pool)
        .await
        .unwrap();

    let all_users = UserExternalID::select_all(&pool).await.unwrap();
    assert_eq!(users, all_users);
}

#[tokio::test]
async fn test_insert_many_internal_ids() {
    let pool = sqlite_mem().await;
    let users = vec![
        UserInternalID::default(),
        UserInternalID::default(),
        UserInternalID::default(),
    ];

    UserInternalID::insert_many(users.clone(), 2, &pool)
        .await
        .unwrap();

    let all_users = UserInternalID::select_all(&pool).await.unwrap();
    assert_eq!(3, all_users.len());
}
