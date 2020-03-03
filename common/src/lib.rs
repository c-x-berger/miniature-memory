#![feature(try_trait)]

mod net;
mod update;

pub use net::{errors::*, Network};
pub use update::UpdateMessage;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
