use color_eyre::eyre::Result as EResult;

pub trait Readable<R> {
    fn read(reader: &mut R) -> EResult<Self>
    where
        Self: Sized;
}
