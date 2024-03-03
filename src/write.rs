use color_eyre::eyre::Result as EResult;

pub trait Writable<W> {
    fn write(&self, writer: &mut W) -> EResult<()>;
}
