//! TODO: docs.

use crate::never::Never;

/// TODO: docs.
pub trait Tuple {
    /// The number of elements in the tuple.
    const LEN: usize;

    /// The type of the first element of the tuple.
    type First;

    /// The type of the last element of the tuple.
    type Last;

    /// The type of the tuple without the first element.
    type FromFirst: Tuple;

    /// The type of the tuple without the last element.
    type UpToLast: Tuple;

    /// TODO: docs.
    type Borrow<'a>: Tuple
    where
        Self: 'a;

    /// Returns a new [`Tuple`] that refers to the elements of this tuple.
    fn borrow(&self) -> Self::Borrow<'_>;

    /// Splits the tuple into its first element and the rest.
    ///
    /// # Panics
    ///
    /// Implementors are expected to panic if the tuple is empty.
    fn split_first(self) -> (Self::First, Self::FromFirst);

    /// Splits the tuple into its last element and the rest.
    ///
    /// # Panics
    ///
    /// Implementors are expected to panic if the tuple is empty.
    fn split_last(self) -> (Self::UpToLast, Self::Last);

    /// Returns whether the tuple is empty (i.e., its [length](Tuple::LEN) is
    /// zero).
    #[inline]
    fn is_empty(&self) -> bool {
        Self::LEN == 0
    }
}

impl Tuple for () {
    const LEN: usize = 0;
    type First = Never;
    type Last = Never;
    type FromFirst = Never;
    type UpToLast = Never;
    type Borrow<'a> = Self;

    #[inline]
    fn borrow(&self) -> Self {}

    #[cold]
    fn split_first(self) -> (Self::First, Self::FromFirst) {
        panic!("Cannot split first element from an empty tuple")
    }
    #[cold]
    fn split_last(self) -> (Self::UpToLast, Self::Last) {
        panic!("Cannot split last element from an empty tuple")
    }
}

impl<T> Tuple for (T,) {
    const LEN: usize = 1;
    type First = T;
    type Last = T;
    type FromFirst = ();
    type UpToLast = ();
    type Borrow<'a>
        = (&'a T,)
    where
        T: 'a;

    #[inline]
    fn borrow(&self) -> Self::Borrow<'_> {
        (&self.0,)
    }
    #[inline]
    fn split_first(self) -> (Self::First, Self::FromFirst) {
        (self.0, ())
    }
    #[inline]
    fn split_last(self) -> (Self::UpToLast, Self::Last) {
        ((), self.0)
    }
}

impl<T> Tuple for [T; 0] {
    const LEN: usize = 0;
    type First = Never;
    type Last = Never;
    type FromFirst = Never;
    type UpToLast = Never;
    type Borrow<'a>
        = ()
    where
        T: 'a;

    #[inline]
    fn borrow(&self) -> Self::Borrow<'_> {}
    #[inline]
    fn split_first(self) -> (Self::First, Self::FromFirst) {
        panic!("Cannot split first element from an empty array")
    }
    #[inline]
    fn split_last(self) -> (Self::UpToLast, Self::Last) {
        panic!("Cannot split last element from an empty array")
    }
}

impl<T> Tuple for [T; 1] {
    const LEN: usize = 1;
    type First = T;
    type Last = T;
    type FromFirst = ();
    type UpToLast = ();
    type Borrow<'a>
        = [&'a T; 1]
    where
        T: 'a;

    #[inline]
    fn borrow(&self) -> Self::Borrow<'_> {
        self.each_ref()
    }

    #[inline]
    fn split_first(self) -> (Self::First, Self::FromFirst) {
        let [first] = self;
        (first, ())
    }
    #[inline]
    fn split_last(self) -> (Self::UpToLast, Self::Last) {
        let [last] = self;
        ((), last)
    }
}

macro_rules! count {
    () => { 0 };
    ($x:tt $($xs:tt)*) => { 1 + count!($($xs)*) };
}

macro_rules! impl_tuple_for_tuple {
    ($($T:ident),*) => {
        impl_tuple_for_tuple!(@pair [] [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31] [$($T)*]);
    };

    // Last element: mark with [] instead of ()
    (@pair [$($pairs:tt)*] [$last_idx:tt $($_:tt)*] [$last_T:ident]) => {
        impl_tuple_for_tuple!(@final [$($pairs)* [$last_idx $last_T]]);
    };

    // Not last: use ()
    (@pair [$($pairs:tt)*] [$next_idx:tt $($rest_idx:tt)*] [$next_T:ident $($rest_T:ident)+]) => {
        impl_tuple_for_tuple!(@pair [$($pairs)* ($next_idx $next_T)] [$($rest_idx)*] [$($rest_T)+]);
    };

    (@final [($first_idx:tt $first_T:ident) $(($mid_idx:tt $mid_T:ident))* [$last_idx:tt $last_T:ident]]) => {
        impl<$first_T, $($mid_T,)* $last_T> Tuple for ($first_T, $($mid_T,)* $last_T,) {
            const LEN: usize = 2 + count!($($mid_T)*);
            type First = $first_T;
            type Last = $last_T;
            type FromFirst = ($($mid_T,)* $last_T,);
            type UpToLast = ($first_T, $($mid_T,)*);
            type Borrow<'a>
                = (&'a $first_T, $(&'a $mid_T,)* &'a $last_T,)
            where
                $first_T: 'a,
                $($mid_T: 'a,)*
                $last_T: 'a;

            #[inline]
            fn borrow(&self) -> Self::Borrow<'_> {
                (&self.$first_idx, $(&self.$mid_idx,)* &self.$last_idx,)
            }

            #[inline]
            fn split_first(self) -> (Self::First, Self::FromFirst) {
                (self.$first_idx, ($(self.$mid_idx,)* self.$last_idx,))
            }

            #[inline]
            fn split_last(self) -> (Self::UpToLast, Self::Last) {
                ((self.$first_idx, $(self.$mid_idx,)*), self.$last_idx)
            }
        }
    };
}

impl_tuple_for_tuple!(T1, T2);
impl_tuple_for_tuple!(T1, T2, T3);
impl_tuple_for_tuple!(T1, T2, T3, T4);
impl_tuple_for_tuple!(T1, T2, T3, T4, T5);
impl_tuple_for_tuple!(T1, T2, T3, T4, T5, T6);
impl_tuple_for_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_tuple_for_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple_for_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_tuple_for_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_tuple_for_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_tuple_for_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_tuple_for_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14
);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16
);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17
);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17,
    T18
);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17,
    T18, T19
);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17,
    T18, T19, T20
);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17,
    T18, T19, T20, T21
);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17,
    T18, T19, T20, T21, T22
);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17,
    T18, T19, T20, T21, T22, T23
);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17,
    T18, T19, T20, T21, T22, T23, T24
);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17,
    T18, T19, T20, T21, T22, T23, T24, T25
);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17,
    T18, T19, T20, T21, T22, T23, T24, T25, T26
);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17,
    T18, T19, T20, T21, T22, T23, T24, T25, T26, T27
);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17,
    T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28
);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17,
    T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29
);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17,
    T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30
);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17,
    T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30, T31
);
impl_tuple_for_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17,
    T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30, T31, T32
);

macro_rules! impl_tuple_for_array {
    ($n:literal) => {
        impl<T> Tuple for [T; $n] {
            const LEN: usize = $n;

            type First = T;
            type Last = T;
            type FromFirst = [T; $n - 1];
            type UpToLast = [T; $n - 1];
            type Borrow<'a>
                = [&'a T; $n]
            where
                T: 'a;

            #[inline]
            fn borrow(&self) -> Self::Borrow<'_> {
                self.each_ref()
            }

            #[inline]
            fn split_first(self) -> (Self::First, Self::FromFirst) {
                unsafe {
                    let arr = core::mem::ManuallyDrop::new(self);
                    let first = core::ptr::read(arr.as_ptr());
                    let rest = core::ptr::read(
                        arr.as_ptr().add(1).cast::<[T; $n - 1]>(),
                    );
                    (first, rest)
                }
            }

            #[inline]
            fn split_last(self) -> (Self::UpToLast, Self::Last) {
                unsafe {
                    let arr = core::mem::ManuallyDrop::new(self);
                    let init =
                        core::ptr::read(arr.as_ptr().cast::<[T; $n - 1]>());
                    let last = core::ptr::read(arr.as_ptr().add($n - 1));
                    (init, last)
                }
            }
        }
    };
}

impl_tuple_for_array!(2);
impl_tuple_for_array!(3);
impl_tuple_for_array!(4);
impl_tuple_for_array!(5);
impl_tuple_for_array!(6);
impl_tuple_for_array!(7);
impl_tuple_for_array!(8);
impl_tuple_for_array!(9);
impl_tuple_for_array!(10);
impl_tuple_for_array!(11);
impl_tuple_for_array!(12);
impl_tuple_for_array!(13);
impl_tuple_for_array!(14);
impl_tuple_for_array!(15);
impl_tuple_for_array!(16);
impl_tuple_for_array!(17);
impl_tuple_for_array!(18);
impl_tuple_for_array!(19);
impl_tuple_for_array!(20);
impl_tuple_for_array!(21);
impl_tuple_for_array!(22);
impl_tuple_for_array!(23);
impl_tuple_for_array!(24);
impl_tuple_for_array!(25);
impl_tuple_for_array!(26);
impl_tuple_for_array!(27);
impl_tuple_for_array!(28);
impl_tuple_for_array!(29);
impl_tuple_for_array!(30);
impl_tuple_for_array!(31);
impl_tuple_for_array!(32);
