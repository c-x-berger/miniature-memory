use std::convert::TryFrom;

pub enum Response {
    OkResponse,
    BadMessage(String),
    BadSignature,
    AlreadyClaimed,
}

impl TryFrom<u16> for Response {
    type Error = String;

    fn try_from(i: u16) -> Result<Self, Self::Error> {
        match i {
            200 => Ok(Self::OkResponse),
            444 => Ok(Self::BadMessage("i haven't the foggiest".to_string())),
            455 => Ok(Self::BadSignature),
            _ => Err(format!("Code {} not understood", i)),
        }
    }
}

pub struct UpdateResponse {
    version: u8,
    status: Response,
}

impl UpdateResponse {
    fn new(version: u8, status: Response) -> Self {
        Self { version, status }
    }
}
