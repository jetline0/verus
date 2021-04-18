extern crate builtin;
use builtin::{assert, assume, imply, int};

fn main() {}

extern {
    #[spec]
    fn f(i: int, j: int) -> bool;
}

fn test1() {
    assert(true);
    assert(!false);
    assert(true && true);
    assert(true || false);
    assert(true && false); // FAILS
    assert(false);
    assert(false);
    assert(true);
}

fn test2(b: bool, x: int, y: int, z: int) {
    assert(b || !b);

    assume(b);
    assert(b);

    assert(imply(x == y, f(x, y) == f(y, x)));

    assert(x + y == y + x);

    assume(x <= y && y <= z);
    assert(x <= z);
    assert(x < z); // FAILS
}
