use crate::tuple::Tuple;

pub enum Never {}

impl Tuple for Never {
    const LEN: usize = unreachable!();
    type First = Self;
    type Last = Self;
    type FromFirst = Self;
    type UpToLast = Self;
    type Borrow<'a> = Self;

    #[cold]
    fn borrow(&self) -> Self {
        unreachable!()
    }

    #[cold]
    fn split_first(self) -> (Self::First, Self::FromFirst) {
        unreachable!()
    }
    #[cold]
    fn split_last(self) -> (Self::UpToLast, Self::Last) {
        unreachable!()
    }
}
