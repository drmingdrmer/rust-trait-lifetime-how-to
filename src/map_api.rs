use std::borrow::Borrow;
use std::future::Future;
use std::ops::RangeBounds;

use futures_util::stream::BoxStream;

use crate::Val;

pub trait MapKey: Clone + Ord + Send + Sync + Unpin + 'static {
    type V: Default + Clone + PartialEq + Send + Sync + Unpin + 'static;
}

impl MapKey for String {
    type V = Val;
}

/// lifetime parameters:
/// - `'d`: the lifetime of the data it references, or itself if it owns the data.
/// - `'rf`: the lifetime of the RangeFuture.
pub trait MapApiRO<'d, 'rf, K>: Send + Sync
where
    K: MapKey,
    'd: 'rf,
{
    type GetFut<'f, Q>: Future<Output = K::V>
    where
        Self: 'f,
        'd: 'f,
        K: Borrow<Q>,
        Q: Ord + Send + Sync + ?Sized,
        Q: 'f;

    fn get<'f, Q>(self, key: &'f Q) -> Self::GetFut<'f, Q>
    where
        K: Borrow<Q>,
        Q: Ord + Send + Sync + ?Sized,
        Q: 'f;

    type RangeFut<Q, R>: Future<Output = BoxStream<'rf, (K, K::V)>>
    where
        K: Borrow<Q>,
        R: RangeBounds<Q> + Send + Sync + Clone,
        Q: Ord + Send + Sync + ?Sized,
        Q: 'rf;

    fn range<Q, R>(self, range: R) -> Self::RangeFut<Q, R>
    where
        K: Borrow<Q>,
        Q: Ord + Send + Sync + ?Sized,
        R: RangeBounds<Q> + Send + Sync + Clone;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait MapApi<'me, 'd, K>: for<'rf> MapApiRO<'d, 'rf, K>
where
    K: MapKey,
    'me: 'd,
{
    type SetFut<'f>: Future<Output = (K::V, K::V)>
    where
        Self: 'f,
        'me: 'f;

    /// Set an entry and returns the old value and the new value.
    fn set<'f>(self, key: K, value: Option<K::V>) -> Self::SetFut<'f>;
}
