//! Generic structs with flattened type parameters: the derived impls carry the bounds they need,
//! so the struct itself does not have to declare any.

use common::sqlite;
use crudly::{
    CrudlyDefault, DeleteById, HasColumns, HasId, Insert, IntoRow, Schema, SelectAll, SelectById,
};
use sqlx::FromRow;

mod common;

#[derive(Debug, Clone, PartialEq, FromRow, IntoRow, Schema)]
#[crudly(table = "keys", external_ids)]
struct Key<Owner, Scope> {
    #[crudly(id)]
    id: i64,
    name: String,
    #[sqlx(flatten)]
    scope: Scope,
    #[sqlx(flatten)]
    owner: Owner,
}
impl<Owner, Scope> CrudlyDefault for Key<Owner, Scope> {}

#[derive(Debug, Clone, PartialEq, FromRow, IntoRow)]
struct UserOwner {
    user_id: i64,
}

#[derive(Debug, Clone, PartialEq, FromRow, IntoRow)]
struct ProjectScope {
    project: String,
    read_only: bool,
}

/// Has no row mapping at all (and is not even `Send`).
#[derive(Debug, PartialEq)]
struct Opaque(std::rc::Rc<str>);

#[test]
fn columns_follow_the_type_arguments() {
    type K = Key<UserOwner, ProjectScope>;
    assert_eq!(<K as Schema>::table_name(), "keys");
    assert_eq!(
        <K as Schema>::columns(),
        vec!["id", "name", "project", "read_only", "user_id"]
    );
    assert_eq!(
        <K as HasColumns>::columns(),
        vec!["name", "project", "read_only", "user_id"]
    );
    assert_eq!(<K as HasId>::id_column(), "id");

    // Swapping the arguments is a different instantiation with its own column order.
    assert_eq!(
        <Key<ProjectScope, UserOwner> as HasColumns>::columns(),
        vec!["name", "user_id", "project", "read_only"]
    );
}

#[tokio::test]
async fn insert_select_delete_round_trip() {
    let pool = sqlite::memory_with_schema(
        "CREATE TABLE keys (
            id INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            project TEXT NOT NULL,
            read_only BOOLEAN NOT NULL,
            user_id INTEGER NOT NULL
        );",
    )
    .await;

    let key = Key {
        id: 7,
        name: "deploy".to_string(),
        scope: ProjectScope {
            project: "website".to_string(),
            read_only: true,
        },
        owner: UserOwner { user_id: 3 },
    };

    key.clone().insert(&pool).await.unwrap();

    let all = Key::<UserOwner, ProjectScope>::select_all(&pool)
        .await
        .unwrap();
    assert_eq!(all, vec![key.clone()]);

    let by_id = Key::<UserOwner, ProjectScope>::select_by_id(&7, &pool)
        .await
        .unwrap();
    assert_eq!(by_id, Some(key));

    assert!(
        Key::<UserOwner, ProjectScope>::delete_by_id(&7, &pool)
            .await
            .unwrap()
    );
    assert!(
        Key::<UserOwner, ProjectScope>::select_all(&pool)
            .await
            .unwrap()
            .is_empty()
    );
}

/// A type argument without any crudly impl is fine as long as no crudly functionality is used.
#[test]
fn usable_as_plain_struct_without_row_mapping() {
    let key = Key {
        id: 1,
        name: "plain".to_string(),
        scope: Opaque("anything".into()),
        owner: UserOwner { user_id: 3 },
    };

    assert_eq!(key.id, 1);
    assert_eq!(key.name, "plain");
    assert_eq!(key.scope, Opaque("anything".into()));
    assert_eq!(key.owner.user_id, 3);

    // The id does not depend on the flattened fields, so `HasId` is still available.
    assert_eq!(HasId::id(&key), 1);
}

/// Bounds written on the struct keep working and are merged with the generated ones.
#[test]
fn bounds_on_the_struct_still_compile() {
    #[derive(IntoRow, Schema)]
    #[crudly(table = "bounded")]
    struct Bounded<T>
    where
        T: HasColumns + Send,
    {
        #[crudly(id)]
        id: i64,
        #[crudly(flatten)]
        inner: T,
    }

    #[derive(IntoRow, Schema)]
    struct InlineBounded<T: HasColumns + Send> {
        label: String,
        #[crudly(flatten)]
        inner: T,
    }

    assert_eq!(
        <Bounded<UserOwner> as Schema>::columns(),
        vec!["id", "user_id"]
    );
    assert_eq!(
        <InlineBounded<UserOwner> as Schema>::columns(),
        vec!["label", "user_id"]
    );
    assert_eq!(
        <InlineBounded<UserOwner> as HasColumns>::columns(),
        vec!["label", "user_id"]
    );
}

/// The id type may be a type parameter as well.
#[test]
fn generic_id_type() {
    #[derive(IntoRow, Schema)]
    #[crudly(table = "things", external_ids)]
    struct Thing<Id> {
        #[crudly(id)]
        id: Id,
        label: String,
    }

    let thing = Thing {
        id: "abc".to_string(),
        label: "x".to_string(),
    };
    assert_eq!(HasId::id(&thing), "abc");
    assert_eq!(<Thing<String> as Schema>::columns(), vec!["id", "label"]);
}
