//! Compile-only mock `sqlx::Database` for tests. All behavior is `unimplemented!`.

use std::fmt::{self, Debug, Display, Formatter};
use std::future::Future;
use std::str::FromStr;

use serde::Serialize;
use sqlx::encode::{Encode, IsNull};
use sqlx::error::BoxDynError;
use sqlx::types::{Json, Type};
use sqlx::{
    Arguments, Column, ColumnIndex, ConnectOptions, Connection, Database, Either, Error, Row,
    SqlStr, Statement, Transaction, TypeInfo, Value, ValueRef,
};
use sqlx_core::transaction::TransactionManager;
use url::Url;

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

    fn connect(&self) -> impl Future<Output = Result<Self::Connection, Error>> + Send + '_ {
        async move { unimplemented!() }
    }

    fn log_statements(self, _level: log::LevelFilter) -> Self {
        unimplemented!()
    }

    fn log_slow_statements(self, _level: log::LevelFilter, _duration: std::time::Duration) -> Self {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct MockConnection;

impl Connection for MockConnection {
    type Database = MockDB;
    type Options = MockConnectOptions;

    fn close(self) -> impl Future<Output = Result<(), Error>> + Send + 'static {
        async move { unimplemented!() }
    }

    fn close_hard(self) -> impl Future<Output = Result<(), Error>> + Send + 'static {
        async move { unimplemented!() }
    }

    fn ping(&mut self) -> impl Future<Output = Result<(), Error>> + Send + '_ {
        async move { unimplemented!() }
    }

    fn begin(
        &mut self,
    ) -> impl Future<Output = Result<Transaction<'_, Self::Database>, Error>> + Send + '_ {
        async move { unimplemented!() }
    }

    fn shrink_buffers(&mut self) {
        unimplemented!()
    }

    fn flush(&mut self) -> impl Future<Output = Result<(), Error>> + Send + '_ {
        async move { unimplemented!() }
    }

    fn should_flush(&self) -> bool {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct MockTransactionManager;

impl TransactionManager for MockTransactionManager {
    type Database = MockDB;

    async fn begin(_conn: &mut MockConnection, _statement: Option<SqlStr>) -> Result<(), Error> {
        unimplemented!()
    }

    fn commit(_conn: &mut MockConnection) -> impl Future<Output = Result<(), Error>> + Send + '_ {
        async move { unimplemented!() }
    }

    fn rollback(_conn: &mut MockConnection) -> impl Future<Output = Result<(), Error>> + Send + '_ {
        async move { unimplemented!() }
    }

    fn start_rollback(_conn: &mut MockConnection) {
        unimplemented!()
    }

    fn get_transaction_depth(_conn: &MockConnection) -> usize {
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

    fn type_info(&self) -> std::borrow::Cow<'_, <Self::Database as Database>::TypeInfo> {
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

    fn type_info(&self) -> std::borrow::Cow<'_, <Self::Database as Database>::TypeInfo> {
        unimplemented!()
    }

    fn is_null(&self) -> bool {
        unimplemented!()
    }
}

/// Bound parameter values as `Display` strings; `None` is SQL `NULL`.
#[derive(Debug, Default)]
pub struct MockArguments {
    pub values: Vec<Option<String>>,
}

impl Arguments for MockArguments {
    type Database = MockDB;

    fn reserve(&mut self, additional: usize, _size: usize) {
        self.values.reserve(additional);
    }

    fn add<'t, T>(&mut self, value: T) -> Result<(), BoxDynError>
    where
        T: Encode<'t, Self::Database> + Type<Self::Database>,
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

sqlx_core::impl_into_arguments_for_arguments!(MockArguments);

#[derive(Debug, Clone)]
pub struct MockStatement {
    sql: SqlStr,
}

impl Statement for MockStatement {
    type Database = MockDB;

    fn into_sql(self) -> SqlStr {
        self.sql
    }

    fn sql(&self) -> &SqlStr {
        &self.sql
    }

    fn parameters(&self) -> Option<Either<&[<Self::Database as Database>::TypeInfo], usize>> {
        unimplemented!()
    }

    fn columns(&self) -> &[<Self::Database as Database>::Column] {
        unimplemented!()
    }

    sqlx_core::impl_statement_query!(MockArguments);
}

impl ColumnIndex<MockStatement> for usize {
    fn index(&self, _container: &MockStatement) -> Result<usize, Error> {
        unimplemented!()
    }
}

impl ColumnIndex<MockStatement> for &str {
    fn index(&self, _container: &MockStatement) -> Result<usize, Error> {
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

impl Encode<'_, MockDB> for String {
    fn encode_by_ref(
        &self,
        buf: &mut <MockDB as Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(Some(self.to_string()));
        Ok(IsNull::No)
    }
}

impl<T> Encode<'_, MockDB> for Option<T>
where
    T: for<'q> Encode<'q, MockDB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <MockDB as Database>::ArgumentBuffer,
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

impl<T: Serialize> Encode<'_, MockDB> for Json<T> {
    fn encode_by_ref(
        &self,
        buf: &mut <MockDB as Database>::ArgumentBuffer,
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
    type Arguments = MockArguments;
    type ArgumentBuffer = Vec<Option<String>>;
    type Statement = MockStatement;

    const NAME: &'static str = "mock";
    const URL_SCHEMES: &'static [&'static str] = &["mock"];
}
