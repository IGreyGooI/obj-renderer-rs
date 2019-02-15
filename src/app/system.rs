use std::{
    sync::{
        RwLock,
        Arc,
    }
};

/// struct implement this trait should be able to be dispatch
/// and work
pub trait Professional {
    fn work(&mut self);
    fn inbox<P, B: Inbox<P>>(&self) -> Arc<RwLock<B>>;
}

pub trait Inbox<P> {
    fn push(&mut self, package: P);
}
