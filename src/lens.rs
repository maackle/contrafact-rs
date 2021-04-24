pub type Lens<O, T> = Box<dyn Fn(&mut O) -> &mut T>;

mod tests {
    use super::*;

    pub struct Tester {
        s: String,
        u: u32,
    }

    impl Tester {
        pub fn s_mut(&mut self) -> &mut String {
            &mut self.s
        }

        pub fn u_mut(&mut self) -> &mut u32 {
            &mut self.u
        }
    }

    pub struct TesterLenses {
        s: Lens<Tester, String>,
        u: Lens<Tester, u32>,
    }

    impl TesterLenses {
        pub fn new() -> Self {
            Self {
                s: Box::new(Tester::s_mut),
                u: Box::new(Tester::u_mut),
            }
        }
    }
}
