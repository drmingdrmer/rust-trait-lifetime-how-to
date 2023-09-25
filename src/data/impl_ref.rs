use std::borrow::Borrow;
use std::future::Future;
use std::ops::RangeBounds;

use futures_util::stream::BoxStream;
use stream_more::KMerge;

use crate::map_api::MapApiRO;
use crate::map_api::MapKey;
use crate::util::by_key_seq;
use crate::Level;
use crate::Ref;
use crate::StaticLevels;

impl<'d> Ref<'d> {
    pub fn new(w: &'d Level, frozen: &'d StaticLevels) -> Self {
        Self {
            writable: w,
            frozen,
        }
    }

    pub fn iter_levels<'a>(&'a self) -> impl Iterator<Item = &'d Level> + 'd {
        [self.writable].into_iter().chain(self.frozen.iter_levels())
    }
}

//////////////////////////////////////////////////////////////////////////////////////////
// &Ref //////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////

impl<'ro_me, 'ro_d, K> MapApiRO<'ro_d, K> for &'ro_me Ref<'ro_d>
where
    K: MapKey,
    for<'him> &'him Level: MapApiRO<'him, K>,
{
    type GetFut<'f, Q> = impl Future<Output =K::V> + 'f
        where Self: 'f,
              'ro_me: 'f,
              'ro_d: 'f,
              K: Borrow<Q>,
              Q: Ord + Send + Sync + ?Sized,
              Q: 'f;

    fn get<'f, Q>(self, key: &'f Q) -> Self::GetFut<'f, Q>
    where
        'ro_me: 'f,
        'ro_d: 'f,
        K: Borrow<Q>,
        Q: Ord + Send + Sync + ?Sized,
    {
        async move {
            for ld in self.iter_levels() {
                let got = ld.get(key).await;
                if got != K::V::default() {
                    return got;
                }
            }
            K::V::default()
        }
    }

    type RangeFut<'f, Q, R> = impl Future<Output = BoxStream<'f, (K, K::V)>>
    where
        // Self: 'f,
        'ro_d: 'f,
        K: Borrow<Q>,
        R: RangeBounds<Q> + Send + Sync + Clone,
        Q: Ord + Send + Sync + ?Sized,
        Q: 'f;

    fn range<'f, Q, R>(self, range: R) -> Self::RangeFut<'f, Q, R>
    where
        // Self: 'f,
        'ro_d: 'f,
        K: Borrow<Q>,
        Q: Ord + Send + Sync + ?Sized,
        R: RangeBounds<Q> + Clone + Send + Sync,
    {
        let levels = self.iter_levels();

        async move {
            let mut km = KMerge::by(by_key_seq);

            for api in levels {
                let a = api.range(range.clone()).await;
                km = km.merge(a);
            }

            let x: BoxStream<'_, (K, K::V)> = Box::pin(km);
            x
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////
// Ref ///////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////

impl<'ro_d, K> MapApiRO<'ro_d, K> for Ref<'ro_d>
where
    K: MapKey,
    for<'him> &'him Level: MapApiRO<'him, K>,
{
    type GetFut<'f, Q> = impl Future<Output =K::V> + 'f
        where Self: 'f,
              'ro_d: 'f,
              K: Borrow<Q>,
              Q: Ord + Send + Sync + ?Sized,
              Q: 'f;

    fn get<'f, Q>(self, key: &'f Q) -> Self::GetFut<'f, Q>
    where
        'ro_d: 'f,
        K: Borrow<Q>,
        Q: Ord + Send + Sync + ?Sized,
    {
        async move { (&self).get(key).await }
    }

    type RangeFut<'f, Q, R> = impl Future<Output = BoxStream<'f, (K, K::V)>>
        where
            Self: 'f,
            'ro_d: 'f,
            K: Borrow<Q>,
            R: RangeBounds<Q> + Send + Sync + Clone,
            Q: Ord + Send + Sync + ?Sized,
            Q: 'f;

    fn range<'f, Q, R>(self, range: R) -> Self::RangeFut<'f, Q, R>
    where
        'ro_d: 'f,
        K: Borrow<Q>,
        Q: Ord + Send + Sync + ?Sized,
        R: RangeBounds<Q> + Clone + Send + Sync,
    {
        // TODO:
        // async move { (&self).range(range).await }

        let levels = self.iter_levels();

        async move {
            let mut km = KMerge::by(by_key_seq);

            for api in levels {
                let a = api.range(range.clone()).await;
                km = km.merge(a);
            }

            let x: BoxStream<'_, (K, K::V)> = Box::pin(km);
            x
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use futures_util::StreamExt;

    use crate::map_api::MapApiRO;
    use crate::Level;
    use crate::Ref;
    use crate::StaticLevels;
    use crate::Val;

    #[tokio::test]
    async fn test_ref() {
        let k = || "a".to_string();

        let mut d = Level {
            kv: Default::default(),
        };

        d.kv.insert(k(), Val(1));

        // &StaticLeveledMap: get
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

        let d = Level {
            kv: Default::default(),
        };

        {
            let r = Ref::new(&d, &static_levels);

            let got = (&r).get(&k()).await;
            assert_eq!(got, Val(2));

            let got = { r }.get(&k()).await;
            assert_eq!(got, Val(2));
        }

        {
            let r = Ref::new(&d, &static_levels);

            let got = (&r).range(k()..).await.collect::<Vec<_>>().await;
            assert_eq!(got, vec![(k(), Val(3)), (k(), Val(2))]);

            let got = { r }.range(k()..).await.collect::<Vec<_>>().await;
            assert_eq!(got, vec![(k(), Val(3)), (k(), Val(2))]);
        }
    }
}
