use color_eyre::eyre::Result as EResult;
use tracing::*;

pub trait Readable<R> {
    fn read(reader: &mut R) -> EResult<Self>
    where
        Self: Sized;
}

#[must_use]
#[instrument(skip(reader, f))]
pub fn read_array<R, T, E>(
    length: u32,
    reader: &mut R,
    f: fn(&mut R) -> Result<T, E>,
) -> Result<Vec<T>, E> {
    (0..length).map(|_| f(reader)).collect()
}
