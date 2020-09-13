#![recursion_limit = "256"]

mod fake_typenum {
    #[derive(Default)]
    pub struct UTerm;

    #[derive(Default)]
    pub struct UInt<H,L>(pub H,pub L);

    #[derive(Default)]
    pub struct B0;

    #[derive(Default)]
    pub struct B1; 
}

#[test]
fn v4() {
    type X1 = ::typenum_uuid::uuid_new_v4!();
    type X2 = ::typenum_uuid::uuid_new_v4!();

    fn assert_is_unsigned<T:typenum::uint::Unsigned>() {}

    assert_is_unsigned::<X1>();
    assert_is_unsigned::<X2>();

    use typenum::uint::Unsigned;
    let x1:u128 = X1::to_u128();
    let x2:u128 = X2::to_u128();

    assert_ne!(x1, x2);

    use typenum::*;

    assert_type!{ op!{ X1 != X2 } };
}

#[test]
fn v4_alternate_typenum() {
    type Y = ::typenum_uuid::uuid_new_v4!(| crate::fake_typenum);

    let y: Y = Default::default();

    #[allow(irrefutable_let_patterns)]
    if let fake_typenum::UInt(_,_) = y {}
    else { panic!("Proc macro didn't use alternate implementation"); }
}

#[test]
fn literal() {
    use ::uuid::Uuid;

    use typenum::Unsigned;
    use ::typenum as local_tn;

    type X0 = ::typenum_uuid::uuid!(a65ff38db5b248d0b03abdf468523d2e);
    type X1 = ::typenum_uuid::uuid!(a65ff38d-b5b2-48d0-b03a-bdf468523d2e | local_tn);
    type X2 = ::typenum_uuid::uuid!(urn:uuid:a65ff38d-b5b2-48d0-b03a-bdf468523d2e);

    let xcmp = Uuid::parse_str("a65ff38d-b5b2-48d0-b03a-bdf468523d2e").unwrap();

    assert_eq!(xcmp.as_u128(), X0::to_u128());
    assert_eq!(xcmp.as_u128(), X1::to_u128());
    assert_eq!(xcmp.as_u128(), X2::to_u128());
}
