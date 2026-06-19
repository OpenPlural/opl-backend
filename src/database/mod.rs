pub mod user;
pub mod session;
pub mod friend;
pub mod member;
pub mod front;
pub mod folder;
pub mod fields;
pub mod privacy;
pub mod deletion;
pub mod time;
pub mod apikey;
pub mod notification;

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use sqlx::mysql::MySqlRow;
use sqlx::{Decode, Executor, MySql, MySqlPool, Row, Type};
use crate::error::WebError;

pub type DatabasePool = Arc<MySqlPool>;
pub type DatabaseResult<T> = Result<T, anyhow::Error>;
pub trait DatabaseExecutor<'a> = Executor<'a, Database = MySql>;

pub fn to_web_error(err: anyhow::Error) -> WebError {
    WebError::DatabaseError(err)
}

pub(in crate::database) fn list_to_map<'a, K, V>(rows: &'a Vec<MySqlRow>, key: &str, value: &str, capacity: usize) -> HashMap<K, Vec<V>>
where
    K: Type<MySql> + Decode<'a, MySql> + Eq + Hash,
    V: Type<MySql> + Decode<'a, MySql> + Ord + Eq + Hash,
{
    let mut map: HashMap<K, Vec<V>> = HashMap::with_capacity(capacity);
    for row in rows {
        let key: K = row.get(key);
        let value: V = row.get(value);

        if let Some(list) = map.get_mut(&key) {
            list.push(value);
        } else {
            let list = vec![value];
            map.insert(key, list);
        }
    }
    map.values_mut().for_each(|list| list.sort());
    map
}