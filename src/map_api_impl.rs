//! Default implementation of MapApiRO for &'ro_me mut T

use std::borrow::Borrow;
use std::ops::RangeBounds;

use crate::map_api::MapApiRO;
use crate::map_api::MapKey;

impl<'ro_me, 'ro_d, K, T> MapApiRO<'ro_d, K> for &'ro_me mut T
where
    K: MapKey,
    &'ro_me T: MapApiRO<'ro_d, K>,
    K: Ord + Send + Sync + 'static,
    T: Send + Sync,
{
    type GetFut<'f, Q> = <&'ro_me T as MapApiRO<'ro_d, K>>::GetFut<'f, Q>
    where
        Self: 'f,
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
        (&*self).get(key)
    }

    type RangeFut<'f, Q, R> = <&'ro_me T as MapApiRO<'ro_d, K>>::RangeFut<'f, Q,R>
    where
        Self: 'f,
        'ro_d: 'f,
        K: Borrow<Q>,
        R: RangeBounds<Q> + Send + Sync + Clone,
        Q: Ord + Send + Sync + ?Sized,
        Q: 'f;

    fn range<'f, Q, R>(self, range: R) -> Self::RangeFut<'f, Q, R>
    where
        Self: 'f,
        'ro_d: 'f,
        K: Borrow<Q>,
        Q: Ord + Send + Sync + ?Sized,
        R: RangeBounds<Q> + Clone + Send + Sync,
    {
        (&*self).range(range)
    }
}
