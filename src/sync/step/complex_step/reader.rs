pub trait Readable<I> where I: Sized + Clone {
    fn iterate(&self) -> dyn Iterator<Item=I>;
}