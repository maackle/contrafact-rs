struct S {
    x: X,
}

struct X;

fn f(s: &mut S) -> &mut X {
    &mut s.x
}
