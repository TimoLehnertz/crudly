use common::sqlite;
use crudly::{
    CrudlyDefault, DeleteAll, InsertManyNoId, InsertNoId, IntoRow, Schema, SelectAllNoId,
};
use sqlx::prelude::FromRow;

mod common;

/// A struct that doesn't have a single id column.
/// Crudly still provides
#[derive(Clone, PartialEq, Debug, FromRow, IntoRow, Schema)]
#[crudly(table = "student_x_course")]
pub struct StudentXCourse {
    student_id: i64,
    course_id: i64,
}

impl CrudlyDefault for StudentXCourse {}

impl StudentXCourse {
    fn new(student_id: i64, course_id: i64) -> Self {
        Self {
            student_id,
            course_id,
        }
    }
}

#[tokio::test]
async fn test_student_x_course() {
    let pool = sqlite::memory_with_schema(
        "CREATE TABLE student_x_course (student_id INTEGER NOT NULL, course_id INTEGER NOT NULL, PRIMARY KEY (student_id, course_id));",
    )
    .await;

    StudentXCourse::new(1, 1).insert(&pool).await.unwrap();

    StudentXCourse::insert_many(vec![StudentXCourse::new(2, 2)], 0, &pool)
        .await
        .unwrap();

    let all = StudentXCourse::select_all(&pool).await.unwrap();
    assert_eq!(
        all,
        vec![StudentXCourse::new(1, 1), StudentXCourse::new(2, 2)]
    );

    StudentXCourse::delete_all(&pool).await.unwrap();

    let all = StudentXCourse::select_all(&pool).await.unwrap();
    assert!(all.is_empty());
}
