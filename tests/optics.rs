use lens_rs::*;

#[derive(Lens)]
struct Complex {
    // #[optic]
    yesno: bool,

    #[optic]
    hairy: Option<(Vec<Option<(String, u8)>>, u8)>,
}

#[test]
fn lens_test() {
    let x = Complex {
        yesno: true,
        // yesno: true,
        hairy: Some((
            vec![
                Some(("a".to_string(), 2u8)),
                None,
                Some(("b".to_string(), 3u8)),
            ],
            4u8,
        )),
    };
    let o = optics!(hairy.Some._1);
    assert_eq!(x.preview_ref(o).unwrap(), &4);
}
