use std::collections::HashMap;

pub fn append<K, V>(map: &mut HashMap<K, Vec<V>>, key: K, value: V) {
    if let Some(list) = map.get_mut(&key) {
        list.push(value);
    } else {
        let list = vec![value];
        map.insert(key, list);
    }
}