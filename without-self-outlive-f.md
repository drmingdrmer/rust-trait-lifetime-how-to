Without `Self: 'f`, there is an error as the following:

`to_ref()` returns `Ref<'ro_me>`,

```rust
    /// Because writable is a mutable reference, we can't return a reference to it.
    /// But instead, we can only reborrow it, with the same lifetime of `self`
    pub fn to_ref<'a>(&'a self) -> Ref<'a> {
        Ref::new(&*self.writable, &self.frozen)
    }
```

```
error: lifetime may not live long enough
  --> src/data/impl_ref_mut.rs:82:9
   |
42 | impl<'ro_me, 'ro_d, K> MapApiRO<'ro_d, K> for &'ro_me RefMut<'ro_d>
   |      ------ lifetime `'ro_me` defined here
...
74 |     fn range<'f, Q, R>(self, range: R) -> Self::RangeFut<'f, Q, R>
   |              -- lifetime `'f` defined here
...
82 |         self.to_ref().range(range)
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^ method was supposed to return data with lifetime `'f` but it is returning data with lifetime `'ro_me`
   |
   = help: consider adding the following bound: `'ro_me: 'f`
```
