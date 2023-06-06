//! Helpers

/// 1MB of pure noise
pub static NOISE: once_cell::sync::Lazy<Vec<u8>> = once_cell::sync::Lazy::new(|| {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    std::iter::repeat_with(|| rng.gen())
        .take(1_000_000)
        .collect()
});

/// 1MB of pure Unstructured noise
pub fn unstructured_noise() -> arbitrary::Unstructured<'static> {
    arbitrary::Unstructured::new(&NOISE)
}
