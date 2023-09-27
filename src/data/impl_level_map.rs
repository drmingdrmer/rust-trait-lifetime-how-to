use std::borrow::Borrow;
use std::future::Future;
use std::ops::RangeBounds;

use futures_util::stream::BoxStream;

use crate::map_api::MapApi;
use crate::map_api::MapApiRO;
use crate::map_api::MapKey;
use crate::Level;
use crate::LevelMap;
use crate::Ref;
use crate::RefMut;
use crate::StaticLevels;

impl LevelMap {
    pub fn new(w: Level, frozen: StaticLevels) -> Self {
        Self {
            writable: w,
            frozen,
        }
    }

    pub fn iter_levels(&self) -> impl Iterator<Item = &Level> {
        [&self.writable].into_iter().chain(self.frozen.iter_levels())
    }

    pub fn to_ref_mut(&mut self) -> RefMut {
        RefMut::new(&mut self.writable, &self.frozen)
    }

    pub fn to_ref(&self) -> Ref {
        Ref::new(&self.writable, &self.frozen)
    }
}

impl<'ro_me, K> MapApiRO<K> for &'ro_me LevelMap
where
    K: MapKey,
    // &'ro_me Level: MapApiRO<K>,
    for<'him> &'him Level: MapApiRO<K>,
{
    type GetFut<'f, Q> = impl Future<Output =K::V> + 'f
        where Self: 'f,
              'ro_me: 'f,
              K: Borrow<Q>,
              Q: Ord + Send + Sync + ?Sized,
              Q: 'f;

    fn get<'f, Q>(self, key: &'f Q) -> Self::GetFut<'f, Q>
    where
        'ro_me: 'f,
        K: Borrow<Q>,
        Q: Ord + Send + Sync + ?Sized,
    {
        self.to_ref().get(key)
    }

    type RangeFut<'f, Q, R> = impl Future<Output = BoxStream<'f, (K, K::V)>>
        where
            Self: 'f,
            'ro_me: 'f,
            K: Borrow<Q>,
            R: RangeBounds<Q> + Send + Sync + Clone,
            Q: Ord + Send + Sync + ?Sized,
            Q: 'f;

    fn range<'f, Q, R>(self, range: R) -> Self::RangeFut<'f, Q, R>
    where
        'ro_me: 'f,
        K: Borrow<Q>,
        Q: Ord + Send + Sync + ?Sized,
        R: RangeBounds<Q> + Clone + Send + Sync,
    {
        self.to_ref().range(range)
    }
}

impl<'me, K> MapApi<'me, K> for &'me mut LevelMap
where
    K: MapKey,
    for<'e> &'e Level: MapApiRO<K>,
    for<'him> &'him mut Level: MapApi<'him, K>,
{
    type SetFut<'f> = impl Future<Output = (K::V, K::V)> + 'f
        where
            Self: 'f,
            'me: 'f;

    fn set<'f>(self, key: K, value: Option<K::V>) -> Self::SetFut<'f>
    where 'me: 'f /*  */

/* 'd: 'f, */ {
        self.to_ref_mut().set(key, value)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use futures_util::StreamExt;

    use crate::map_api::MapApi;
    use crate::map_api::MapApiRO;
    use crate::Level;
    use crate::LevelMap;
    use crate::StaticLevels;
    use crate::Val;

    #[tokio::test]
    async fn test_level_map() {
        let k = || "a".to_string();

        let mut d = Level {
            kv: Default::default(),
        };

        d.kv.insert(k(), Val(1));

        let static_levels = {
            let mut d1 = Level {
                kv: Default::default(),
            };

            let mut d2 = Level {
                kv: Default::default(),
            };

            d1.kv.insert(k(), Val(3));
            d2.kv.insert(k(), Val(2));

            StaticLevels::new([Arc::new(d1), Arc::new(d2)])
        };

        {
            let d = Level {
                kv: Default::default(),
            };
            let mut lm = LevelMap::new(d, static_levels);

            let res = lm.set(k(), Some(Val(7))).await;
            assert_eq!(res, (Val(2), Val(7)));

            let got = lm.get(&k()).await;
            assert_eq!(got, Val(7));

            let strm = lm.range(k()..).await;
            let got = strm.collect::<Vec<_>>().await;
            assert_eq!(got, vec![(k(), Val(2)), (k(), Val(7)), (k(), Val(3))]);

            // let x = k();
            // let fu = lm.get(x.as_str());
            //
            // let fu = foo(fu);
            //
            // fn foo<T: Send>(v: T) -> T {
            //     v
            // }
        }
    }
}
