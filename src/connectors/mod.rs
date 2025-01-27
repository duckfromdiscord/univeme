use std::error::Error;

pub mod ledfx;
pub mod pprefox;
#[cfg(windows)]
pub mod windows;
#[cfg(windows)]
pub mod wpeng;

#[async_trait::async_trait]
pub trait Connector
where
    Self: Sized,
{
    fn new() -> Result<Self, Box<dyn Error + 'static>>;
    fn verify(&self) -> Result<(), Box<dyn Error + 'static>>;
    async fn apply(&self) -> Result<(), Box<dyn Error + 'static>>;
}
