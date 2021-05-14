# contrafact-rs

![CI](https://github.com/maackle/contrafact-rs/actions/workflows/rust.yml/badge.svg)

A framework for defining highly composable constraints which can be used both for verification and for generating arbitrary data which satisfies the constraints

## TODO

- [ ] write readme
- [ ] add `Fact::label` method
- [ ] add ability to short-circuit a failing Check
- [ ] consider using [lens-rs](https://github.com/TOETOE55/lens-rs) for optics instead of closures over mut refs
    - this would enable use of immutable data,
    - which would enable things like prisms which construct a value that's derivable from, but not explicitly present in all, variants of an enum
