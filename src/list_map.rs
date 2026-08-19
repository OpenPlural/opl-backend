use std::collections::HashMap;
use std::hash::Hash;

pub fn append<K, V>(map: &mut HashMap<K, Vec<V>>, key: K, value: V)
where
    K: Eq + Hash,
{
    if let Some(list) = map.get_mut(&key) {
        list.push(value);
    } else {
        let list = vec![value];
        map.insert(key, list);
    }
}