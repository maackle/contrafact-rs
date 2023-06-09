//! Helpers

use crate::fact::Generator;

/// 1MB of pure noise
pub static NOISE: once_cell::sync::Lazy<Vec<u8>> = once_cell::sync::Lazy::new(|| {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    std::iter::repeat_with(|| rng.gen())
        .take(1_000_000)
        .collect()
});

/// 1MB of pure Unstructured noise
fn unstructured_noise() -> arbitrary::Unstructured<'static> {
    arbitrary::Unstructured::new(&NOISE)
}

/// A generator backed by 1M of randomness. Useful for tests and for examples.
/// This should not be used in production tests. Better to use a fuzzer like AFL
/// to generate bytes, and construct a generator using `Generator::from(bytes)`
pub fn random_generator() -> Generator<'static> {
    unstructured_noise().into()
}
