#![feature(try_trait)]

mod net;
mod update;
mod update_response;

pub use net::{errors::*, Network};
pub use update::UpdateMessage;
pub use update_response::{Response, UpdateResponse};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
