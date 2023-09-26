use std::borrow::Borrow;
use std::future::Future;
use std::ops::RangeBounds;

use futures_util::stream::BoxStream;

use crate::map_api::MapApi;
use crate::map_api::MapApiRO;
use crate::map_api::MapKey;
use crate::Level;
use crate::Ref;
use crate::RefMut;
use crate::StaticLevels;

impl<'d> RefMut<'d> {
    pub fn new(w: &'d mut Level, frozen: &'d StaticLevels) -> Self {
        Self {
            writable: w,
            frozen,
        }
    }

    pub fn iter_levels(&self) -> impl Iterator<Item = &Level> {
        [&*self.writable].into_iter().chain(self.frozen.iter_levels())
    }

    /// Because writable is a mutable reference, we can't return a reference to it.
    /// But instead, we can only reborrow it, with the same lifetime of `self`
    pub fn to_ref<'a>(&'a self) -> Ref<'a> {
        Ref::new(&*self.writable, &self.frozen)
    }

    pub fn into_ref(self) -> Ref<'d> {
        Ref::new(self.writable, self.frozen)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////
// &RefMut ///////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////

impl<'ro_me, 'ro_d, 'ro_rf, K> MapApiRO<'ro_d, 'ro_rf, K> for &'ro_me RefMut<'ro_d>
where
    K: MapKey,
    Self: 'ro_rf,
    // Self: 'ro_d,
    'ro_d: 'ro_rf,

    &'ro_d Level: MapApiRO<'ro_d, 'ro_rf, K>,
    // for<'him> &'him Level: MapApiRO<'him, 'ro_rf, K>,
    // for<'him, 'him_rf> &'him Level: MapApiRO<'him, 'him_rf, K>,
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
        self.to_ref().get(key)
    }

    type RangeFut<Q, R> = impl Future<Output = BoxStream<'ro_rf, (K, K::V)>>
        where
            K: Borrow<Q>,
            R: RangeBounds<Q> + Send + Sync + Clone,
            Q: Ord + Send + Sync + ?Sized,
            Q: 'ro_rf;

    fn range<Q, R>(self, range: R) -> Self::RangeFut<Q, R>
    where
        K: Borrow<Q>,
        Q: Ord + Send + Sync + ?Sized,
        R: RangeBounds<Q> + Clone + Send + Sync,
    {
        self.to_ref().range(range)
    }
}

impl<'me, 'd, 'rf, K> MapApi<'me, 'd, 'rf, K> for &'me mut RefMut<'d>
where
    K: MapKey,
    'me: 'rf,
    // 'me: 'd,
    'd: 'rf,
    for<'e, 'e_rf> &'e Level: MapApiRO<'e, 'e_rf, K>,
    for<'him, 'him_rf> &'him mut Level: MapApi<'him, 'him, 'him_rf, K>,
{
    type SetFut<'f> = impl Future<Output = (K::V, K::V)> + 'f
    where
        Self: 'f,
        'me: 'f;

    fn set<'f>(self, key: K, value: Option<K::V>) -> Self::SetFut<'f>
    where
        'me: 'f,
        'd: 'f,
    {
        async move {
            let prev = self.get(&key).await;

            let (_prev, res) = self.writable.set(key.clone(), value).await;
            (prev, res)
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////
// RefMut ////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////

impl<'ro_d, 'ro_rf, K> MapApiRO<'ro_d, 'ro_rf, K> for RefMut<'ro_d>
where
    K: MapKey,
    'ro_d: 'ro_rf,
    for<'him, 'him_rf> &'him Level: MapApiRO<'him, 'him_rf, K>,
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
        self.into_ref().get(key)
    }

    type RangeFut<Q, R> = impl Future<Output = BoxStream<'ro_rf, (K, K::V)>>
    where
        K: Borrow<Q>,
        R: RangeBounds<Q> + Send + Sync + Clone,
        Q: Ord + Send + Sync + ?Sized,
        Q: 'ro_rf;

    fn range<Q, R>(self, range: R) -> Self::RangeFut<Q, R>
    where
        K: Borrow<Q>,
        Q: Ord + Send + Sync + ?Sized,
        R: RangeBounds<Q> + Clone + Send + Sync,
    {
        self.into_ref().range(range)
    }
}

impl<'me, 'd, 'rf, K> MapApi<'me, 'd, 'rf, K> for RefMut<'d>
where
    K: MapKey,
    'me: 'd,
    'd: 'rf,
    for<'e, 'e_rf> &'e Level: MapApiRO<'e, 'e_rf, K>,
    for<'him, 'him_rf> &'him mut Level: MapApi<'him, 'him, 'him_rf, K>,
{
    type SetFut<'f> = impl Future<Output = (K::V, K::V)> + 'f
        where
            Self: 'f,
            'me: 'f;

    fn set<'f>(self, key: K, value: Option<K::V>) -> Self::SetFut<'f>
    where
        'me: 'f,
        'd: 'f,
    {
        async move {
            let prev = (&self).get(&key).await;

            let (_prev, res) = self.writable.set(key.clone(), value).await;
            (prev, res)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::map_api::MapApi;
    use crate::map_api::MapApiRO;
    use crate::Level;
    use crate::RefMut;
    use crate::StaticLevels;
    use crate::Val;

    #[tokio::test]
    async fn test_ref_mut() {
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

        // RefMut :: get()
        {
            let mut d = Level {
                kv: Default::default(),
            };

            let mut rm = RefMut::new(&mut d, &static_levels);
            let got = (&rm).get(&k()).await;
            println!("LeveledRefMut: {:?}", got);

            let res = (&mut rm).set(k(), Some(Val(5))).await;
            println!("LeveledRefMut::set() res: {:?}", res);

            // let fu = { rm }.get(&k());
            // let fu = foo(fu);
            // fn foo<T: Send>(v: T) -> T {
            //     v
            // }

            let got = { rm }.get(&k()).await;
            println!("LeveledRefMut: {:?}", got);
        }
    }
}
