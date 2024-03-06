use color_eyre::eyre::Result as EResult;
use tracing::*;

pub trait Writable<W> {
    fn write(&self, writer: &mut W) -> EResult<()>;
}

#[instrument(name = "write_array", skip_all, fields(len = array.len()))]
pub fn write_array<W, T, E>(
    writer: &mut W,
    array: &[T],
    mut f: impl FnMut(&mut W, &T) -> Result<(), E>,
) -> Result<(), E> {
    for item in array {
        f(writer, item)?;
    }
    Ok(())
}
