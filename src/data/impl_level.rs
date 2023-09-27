use std::borrow::Borrow;
use std::future::Future;
use std::ops::RangeBounds;

use futures_util::stream::BoxStream;
use futures_util::StreamExt;

use crate::map_api::MapApi;
use crate::map_api::MapApiRO;
use crate::map_api::MapKey;
use crate::Level;

impl<'d> MapApiRO<String> for &'d Level {
    type GetFut<'f, Q> = impl Future<Output =<String as MapKey>::V> + 'f
        where
            Self: 'f,
            String: Borrow<Q>,
            Q: Ord + Send + Sync + ?Sized,
            Q: 'f;

    fn get<'f, Q>(self, key: &'f Q) -> Self::GetFut<'f, Q>
    where
        String: Borrow<Q>,
        Q: Ord + Send + Sync + ?Sized,
        Q: 'f,
    {
        async move { self.kv.get(key).cloned().unwrap_or_default() }
    }

    type RangeFut<'f, Q, R> = impl Future<Output = BoxStream<'f, (String, <String as MapKey>::V)>>
        where
            Self: 'f,
            String: Borrow<Q>,
            R: RangeBounds<Q> + Send + Sync + Clone,
            Q: Ord + Send + Sync + ?Sized,
            Q: 'f;

    fn range<'f, Q, R>(self, range: R) -> Self::RangeFut<'f, Q, R>
    where
        String: Borrow<Q>,
        Q: Ord + Send + Sync + ?Sized,
        R: RangeBounds<Q> + Clone + Send + Sync,
    {
        async move {
            let it = self.kv.range(range).map(|(k, v)| (k.clone(), v.clone()));
            futures::stream::iter(it).boxed()
        }
    }
}

impl<'me> MapApi<'me, String> for &'me mut Level {
    type SetFut<'f> = impl Future<Output = (<String as MapKey>::V, <String as MapKey>::V)> + 'f
        where
            Self: 'f,
    ;

    fn set<'f>(
        self,
        key: String,
        value: Option<<String as MapKey>::V>,
    ) -> Self::SetFut<'f>
    where
        'me: 'f,
    {
        async move {
            let prev = self.kv.insert(key.clone(), value.unwrap());
            (
                prev.unwrap_or_default(),
                self.kv.get(&key).cloned().unwrap_or_default(),
            )
        }
    }
}

// This will fail, with lifetime `'d`
// impl<'me, 'd> MapApi<'me, 'd, String> for &'me mut Level {
//     type SetFut<'f> = impl Future<Output = (<String as MapKey>::V, <String as
// MapKey>::V)> + 'f         where
//             Self: 'f,
//             'me : 'f
//     ;
//
//     fn set<'f>(self, key: String, value: Option<<String as MapKey>::V>) ->
// Self::SetFut<'f>         where
//             'me: 'f,
//     {
//         async move {
//             let prev = self.kv.insert(key.clone(), value.unwrap());
//             (
//                 prev.unwrap_or_default(),
//                 self.kv.get(&key).cloned().unwrap_or_default(),
//             )
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use crate::map_api::MapApi;
    use crate::map_api::MapApiRO;
    use crate::util::assert_send;
    use crate::Level;
    use crate::Val;

    #[tokio::test]
    async fn test_level() {
        let k = || "a".to_string();

        let mut d = Level {
            kv: Default::default(),
        };

        d.kv.insert(k(), Val(1));

        let got = d.get(&k()).await;
        assert_eq!(got, Val(1));

        let res = d.set(k(), Some(Val(2))).await;
        assert_eq!(res, (Val(1), Val(2)));

        let got = d.get(&k()).await;
        assert_eq!(got, Val(2));

        /////////////

        let x = k();
        let fu = d.get(x.as_str());
        let fu = assert_send(fu);
    }
}
