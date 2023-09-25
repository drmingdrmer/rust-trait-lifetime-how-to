use crate::map_api::MapKey;

pub fn by_key_seq<K, V>((k1, _v1): &(K, V), (k2, _v2): &(K, V)) -> bool
where K: MapKey {
    k1 <= k2
}
