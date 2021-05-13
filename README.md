# contrafact

Readme coming soon.

## TODO

- [ ] write readme
- [ ] add reason for `never()`
- [ ] add `Fact::label` method
- [ ] add ability to short-circuit a failing Check
- [ ] consider using [lens-rs](https://github.com/TOETOE55/lens-rs) for optics instead of closures over mut refs
    - this would enable use of immutable data,
    - which would enable things like prisms which construct a value that's derivable from, but not explicitly present in all, variants of an enum
