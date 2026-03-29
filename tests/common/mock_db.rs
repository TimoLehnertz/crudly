//! Compile-only mock `sqlx::Database` for tests. All behavior is `unimplemented!`.

use std::fmt::{self, Debug, Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;

use std::borrow::Cow;
use std::marker::PhantomData;

use serde::Serialize;
use sqlx::encode::{Encode, IsNull};
use sqlx::error::BoxDynError;
use sqlx::types::{Json, Type};
use sqlx::{
    Arguments, Column, ColumnIndex, ConnectOptions, Connection, Database, Either, Error, IntoArguments,
    Row, Statement, Transaction, TransactionManager, TypeInfo, Value, ValueRef,
};
use url::Url;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct MockConnectOptions;

impl FromStr for MockConnectOptions {
    type Err = Error;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        unimplemented!()
    }
}

impl ConnectOptions for MockConnectOptions {
    type Connection = MockConnection;

    fn from_url(_url: &Url) -> Result<Self, Error> {
        unimplemented!()
    }

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, Error>> {
        Box::pin(async move { unimplemented!() })
    }

    fn log_statements(self, _level: log::LevelFilter) -> Self {
        unimplemented!()
    }

    fn log_slow_statements(
        self,
        _level: log::LevelFilter,
        _duration: std::time::Duration,
    ) -> Self {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct MockConnection;

impl Connection for MockConnection {
    type Database = MockDB;
    type Options = MockConnectOptions;

    fn close(self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move { unimplemented!() })
    }

    fn close_hard(self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move { unimplemented!() })
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move { unimplemented!() })
    }

    fn begin(
        &mut self,
    ) -> BoxFuture<'_, Result<Transaction<'_, Self::Database>, Error>> {
        Box::pin(async move { unimplemented!() })
    }

    fn shrink_buffers(&mut self) {
        unimplemented!()
    }

    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move { unimplemented!() })
    }

    fn should_flush(&self) -> bool {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct MockTransactionManager;

impl TransactionManager for MockTransactionManager {
    type Database = MockDB;

    fn begin<'conn>(
        _conn: &'conn mut <Self::Database as Database>::Connection,
        _statement: Option<Cow<'static, str>>,
    ) -> BoxFuture<'conn, Result<(), Error>> {
        Box::pin(async move { unimplemented!() })
    }

    fn commit(_conn: &mut <Self::Database as Database>::Connection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move { unimplemented!() })
    }

    fn rollback(_conn: &mut <Self::Database as Database>::Connection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move { unimplemented!() })
    }

    fn start_rollback(_conn: &mut <Self::Database as Database>::Connection) {
        unimplemented!()
    }

    fn get_transaction_depth(_conn: &<Self::Database as Database>::Connection) -> usize {
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MockTypeInfo;

impl Display for MockTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "MockTypeInfo")
    }
}

impl TypeInfo for MockTypeInfo {
    fn is_null(&self) -> bool {
        unimplemented!()
    }

    fn name(&self) -> &str {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct MockColumn;

impl Column for MockColumn {
    type Database = MockDB;

    fn ordinal(&self) -> usize {
        unimplemented!()
    }

    fn name(&self) -> &str {
        unimplemented!()
    }

    fn type_info(&self) -> &<Self::Database as Database>::TypeInfo {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct MockRow;

impl Row for MockRow {
    type Database = MockDB;

    fn columns(&self) -> &[<Self::Database as Database>::Column] {
        unimplemented!()
    }

    fn try_get_raw<I>(&self, _index: I) -> Result<<Self::Database as Database>::ValueRef<'_>, Error>
    where
        I: ColumnIndex<Self>,
    {
        unimplemented!()
    }
}

impl ColumnIndex<MockRow> for usize {
    fn index(&self, _container: &MockRow) -> Result<usize, Error> {
        unimplemented!()
    }
}

impl ColumnIndex<MockRow> for &str {
    fn index(&self, _container: &MockRow) -> Result<usize, Error> {
        unimplemented!()
    }
}

#[derive(Debug, Default)]
pub struct MockQueryResult;

impl Extend<MockQueryResult> for MockQueryResult {
    fn extend<T: IntoIterator<Item = MockQueryResult>>(&mut self, _iter: T) {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct MockValue;

impl Value for MockValue {
    type Database = MockDB;

    fn as_ref(&self) -> <Self::Database as Database>::ValueRef<'_> {
        unimplemented!()
    }

    fn type_info(&self) -> Cow<'_, <Self::Database as Database>::TypeInfo> {
        unimplemented!()
    }

    fn is_null(&self) -> bool {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct MockValueRef<'r> {
    pub(crate) _marker: std::marker::PhantomData<&'r ()>,
}

impl<'r> ValueRef<'r> for MockValueRef<'r> {
    type Database = MockDB;

    fn to_owned(&self) -> <Self::Database as Database>::Value {
        unimplemented!()
    }

    fn type_info(&self) -> Cow<'_, <Self::Database as Database>::TypeInfo> {
        unimplemented!()
    }

    fn is_null(&self) -> bool {
        unimplemented!()
    }
}

/// Bound parameter values as `Display` strings; `None` is SQL `NULL`.
#[derive(Debug, Default)]
pub struct MockArguments<'q> {
    pub values: Vec<Option<String>>,
    _marker: PhantomData<&'q ()>,
}

impl<'q> Arguments<'q> for MockArguments<'q> {
    type Database = MockDB;

    fn reserve(&mut self, additional: usize, _size: usize) {
        self.values.reserve(additional);
    }

    fn add<T>(&mut self, value: T) -> Result<(), BoxDynError>
    where
        T: 'q + Encode<'q, Self::Database> + Type<Self::Database>,
    {
        let len_before = self.values.len();
        match value.encode(&mut self.values) {
            Ok(IsNull::Yes) => self.values.push(None),
            Ok(IsNull::No) => {}
            Err(e) => {
                self.values.truncate(len_before);
                return Err(e);
            }
        }
        Ok(())
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

impl<'q> IntoArguments<'q, MockDB> for MockArguments<'q> {
    fn into_arguments(self) -> <MockDB as Database>::Arguments<'q> {
        self
    }
}

#[derive(Debug)]
pub struct MockStatement<'q> {
    _marker: std::marker::PhantomData<&'q ()>,
}

impl<'q> Statement<'q> for MockStatement<'q> {
    type Database = MockDB;

    fn to_owned(&self) -> <Self::Database as Database>::Statement<'static> {
        unimplemented!()
    }

    fn sql(&self) -> &str {
        unimplemented!()
    }

    fn parameters(&self) -> Option<Either<&[<Self::Database as Database>::TypeInfo], usize>> {
        unimplemented!()
    }

    fn columns(&self) -> &[<Self::Database as Database>::Column] {
        unimplemented!()
    }

    fn query(&self) -> sqlx::query::Query<'_, Self::Database, <Self::Database as Database>::Arguments<'_>> {
        unimplemented!()
    }

    fn query_with<'s, A>(&'s self, _arguments: A) -> sqlx::query::Query<'s, Self::Database, A>
    where
        A: IntoArguments<'s, Self::Database>,
    {
        unimplemented!()
    }

    fn query_as<O>(
        &self,
    ) -> sqlx::query::QueryAs<
        '_,
        Self::Database,
        O,
        <Self::Database as Database>::Arguments<'_>,
    >
    where
        O: for<'r> sqlx::FromRow<'r, <Self::Database as Database>::Row>,
    {
        unimplemented!()
    }

    fn query_as_with<'s, O, A>(
        &'s self,
        _arguments: A,
    ) -> sqlx::query::QueryAs<'s, Self::Database, O, A>
    where
        O: for<'r> sqlx::FromRow<'r, <Self::Database as Database>::Row>,
        A: IntoArguments<'s, Self::Database>,
    {
        unimplemented!()
    }

    fn query_scalar<O>(
        &self,
    ) -> sqlx::query::QueryScalar<
        '_,
        Self::Database,
        O,
        <Self::Database as Database>::Arguments<'_>,
    >
    where
        (O,): for<'r> sqlx::FromRow<'r, <Self::Database as Database>::Row>,
    {
        unimplemented!()
    }

    fn query_scalar_with<'s, O, A>(
        &'s self,
        _arguments: A,
    ) -> sqlx::query::QueryScalar<'s, Self::Database, O, A>
    where
        (O,): for<'r> sqlx::FromRow<'r, <Self::Database as Database>::Row>,
        A: IntoArguments<'s, Self::Database>,
    {
        unimplemented!()
    }
}

impl ColumnIndex<MockStatement<'_>> for usize {
    fn index(&self, _container: &MockStatement<'_>) -> Result<usize, Error> {
        unimplemented!()
    }
}

impl ColumnIndex<MockStatement<'_>> for &str {
    fn index(&self, _container: &MockStatement<'_>) -> Result<usize, Error> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct MockDB;

impl Type<MockDB> for String {
    fn type_info() -> MockTypeInfo {
        MockTypeInfo
    }
}

impl<'q> Encode<'q, MockDB> for String {
    fn encode_by_ref(
        &self,
        buf: &mut <MockDB as Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(Some(format!("{self}")));
        Ok(IsNull::No)
    }
}

impl<'q, T> Encode<'q, MockDB> for Option<T>
where
    T: Encode<'q, MockDB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <MockDB as Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        match self {
            None => Ok(IsNull::Yes),
            Some(value) => value.encode_by_ref(buf),
        }
    }
}

impl<T> Type<MockDB> for Json<T> {
    fn type_info() -> MockTypeInfo {
        MockTypeInfo
    }
}

impl<'q, T: Serialize> Encode<'q, MockDB> for Json<T> {
    fn encode_by_ref(
        &self,
        buf: &mut <MockDB as Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        let s = serde_json::to_string(self).map_err(BoxDynError::from)?;
        buf.push(Some(s));
        Ok(IsNull::No)
    }
}

impl Database for MockDB {
    type Connection = MockConnection;
    type TransactionManager = MockTransactionManager;
    type Row = MockRow;
    type QueryResult = MockQueryResult;
    type Column = MockColumn;
    type TypeInfo = MockTypeInfo;
    type Value = MockValue;
    type ValueRef<'r> = MockValueRef<'r>;
    type Arguments<'q> = MockArguments<'q>;
    type ArgumentBuffer<'q> = Vec<Option<String>>;
    type Statement<'q> = MockStatement<'q>;

    const NAME: &'static str = "mock";
    const URL_SCHEMES: &'static [&'static str] = &["mock"];
}
