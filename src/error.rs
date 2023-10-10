use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Network(#[from] reqwest::Error),
    #[error(transparent)]
    JWT(#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    HexDecode(#[from] hex::FromHexError),
}
