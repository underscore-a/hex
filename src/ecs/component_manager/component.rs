use super::AsAny;
use crate::cid;

pub trait Component {
    fn id() -> usize {
        println!("here");

        cid!()
    }
}

impl<'a, C> AsAny<'a> for C where C: Component + 'a {}
