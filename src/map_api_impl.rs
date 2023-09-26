//! Default implementation of MapApiRO for &'ro_me mut T

use std::borrow::Borrow;
use std::ops::RangeBounds;

use crate::map_api::MapApiRO;
use crate::map_api::MapKey;

impl<'ro_me, 'ro_d, 'ro_rf, K, T> MapApiRO<'ro_d, 'ro_rf, K> for &'ro_me mut T
where
    K: MapKey,
    'ro_d: 'ro_rf,
    &'ro_me T: MapApiRO<'ro_d, 'ro_rf, K>,
    K: Ord + Send + Sync + 'static,
    T: Send + Sync,
{
    type GetFut<'f, Q> = <&'ro_me T as MapApiRO<'ro_d, 'ro_rf, K>>::GetFut<'f, Q>
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

    type RangeFut<Q, R> = <&'ro_me T as MapApiRO<'ro_d, 'ro_rf, K>>::RangeFut<Q,R>
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
        (&*self).range(range)
    }
}
