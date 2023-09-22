use std::collections::BTreeMap;

pub fn group_by<I, T, F, K>(v: I, key: F) -> BTreeMap<T, Vec<K>>
where
    I: IntoIterator<Item = K>,
    T: Ord,
    F: Fn(&K) -> T,
{
    let mut map: BTreeMap<T, Vec<K>> = BTreeMap::new();
    for item in v.into_iter() {
        let key = key(&item);
        map.entry(key).or_default().push(item)
    }
    return map;
}
