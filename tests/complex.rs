// dhtop
enum Z {
    A(X, Y),
    B(X),
}

// header
enum X {
    A(Y),
    B,
}

// entry
struct Y {
    id: u32,
}

struct IdRangeFact {
    lo: u32,
    hi: u32,
}

struct ZFact;
