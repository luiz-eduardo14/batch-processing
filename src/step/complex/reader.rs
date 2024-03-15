pub trait Readable<I> {
    fn iterate(&self) -> I;
}