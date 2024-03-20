pub trait Writable<O> {
    fn write(&self, output: &Vec<O>);
}