use flusty::flusty;

#[flusty]
pub fn do_nothing() {}

#[flusty]
pub struct Foo {
    pub x: i32,
}

#[flusty]
pub enum Bar {
    A,
    B,
    C,
}

fn main() {
    do_nothing();
}
