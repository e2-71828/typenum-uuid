//! A crate to generate typesystem-level UUIDs
//!
//! By default, all of the macros produce `::typenum::Unsigned`
//! types.
//! If `typenum` is located somewhere else, you can append
//! `| path::to::typenum` to the macro argument, and it will use
//! that prefix instead:
//! ```
//! # use typenum_uuid::uuid_new_v4;
//! use ::typenum as some_other_name;
//! type ID = uuid_new_v4!( | some_other_name );
//! ```
//! This feature is most useful when exporting macros from a crate
//! that relies on `typenum`: the UUIDs can be made to use your
//! crate's re-export of `typenum`, in case your users have an
//! incompatible version.

use uuid::Uuid;
use std::iter;

extern crate proc_macro;
use proc_macro::*;

/// Appends an identifier to a type path
///
/// Roughly equivalent to the macro_rules expansion:
/// ```ignore
/// $prefix :: $id
/// ```
fn prefixed_ident(prefix: &TokenStream, id: &str)->impl Iterator<Item=TokenTree> {
    prefix.clone().into_iter().chain(
        vec![
            Punct::new(':', Spacing::Joint).into(),
            Punct::new(':', Spacing::Alone).into(),
            Ident::new(id, Span::call_site()).into()
        ].into_iter()
    )
}

/// A mirror of how `typenum` describes unsigned integers:
/// recursively with the least significant bit in the outermost
/// type
enum TypenumUint {
    Lsb(Box<TypenumUint>, bool),
    Term,
}

impl From<u128> for TypenumUint {
    fn from(x:u128)->Self {
        if x == 0 { return Self::Term; }
        else { Self::Lsb( Box::new(Self::from(x >> 1)), (x & 1) != 0 ) }
    }
}

impl TypenumUint {
    /// Write `self` into `ts`.
    ///
    /// `prefix` is the location of the `typenum` crate.
    fn write_ts(&self, prefix: &TokenStream, ts: &mut TokenStream) {
        match self {
            Self::Term => ts.extend(prefixed_ident(prefix, "UTerm")),
            Self::Lsb(high, bit) => {
                ts.extend(prefixed_ident(prefix, "UInt"));
                ts.extend(iter::once::<TokenTree>(
                    Punct::new('<', Spacing::Alone).into()
                ));
                high.write_ts(prefix, ts);
                ts.extend(iter::once::<TokenTree>(
                    Punct::new(',', Spacing::Alone).into()
                ));
                ts.extend(prefixed_ident(prefix, if *bit { "B1" } else { "B0" }));
                ts.extend(iter::once::<TokenTree>(
                    Punct::new('>', Spacing::Alone).into()
                ));
            }
        }
    }
}

/// Convert a Uuid object into a TokenStream
///
/// The resulting stream contains a type declaration that implements
/// `typenum::Unsigned`.  If the `i128` feature of `typenum` is enabled,
/// `<result as Unsigned>::to_u128()` will equal `uuid.as_u128()`
///
/// `prefix` should be the path to the `typenum` crate at the macro
/// expansion point.
fn uuid_to_tokenstream(uuid: Uuid, prefix: TokenStream)->TokenStream {
    let mut result = TokenStream::new();
    TypenumUint::from(uuid.as_u128()).write_ts(&prefix, &mut result);
    result
}

/// Separate local from global macro arguments.
///
/// The macros in this crate all allow `| path::to::typenum` to be
/// appended to the regular arguments in order to specify where to
/// find `typenum`.  This function is responsible for finding and
/// interpreting this, and using the default value of `::typenum`
/// if none is given.
fn split_off_prefix(args: TokenStream) -> (TokenStream, TokenStream) {
    let mut args = args.into_iter();
    let local = (&mut args).take_while(
        |tt| match tt {
            TokenTree::Punct(ref p) if p.as_char() == '|' => false,
            _ => true
        }
    ).collect();
    let mut prefix:TokenStream = args.collect();
    if prefix.is_empty() {
        let x:Vec<TokenTree> = vec![
            Punct::new(':', Spacing::Joint).into(),
            Punct::new(':', Spacing::Alone).into(),
            Ident::new("typenum", Span::call_site()).into()
        ];
        prefix = x.into_iter().collect();
    }
    (local, prefix)
}

/// Construct a new random UUID
///
/// This macro constructs a new random (v4) UUID at compile
/// time and returns it as a `typenum::Unsigned` type.
/// Tis can be used to enable the type system to perform
/// a limited form of negative reasoning on type identities.
///
/// For example:
/// ```
/// #![recursion_limit = "256"]
/// use typenum::{Unsigned,IsEqual,True,False};
/// use typenum_uuid::uuid_new_v4;
///
/// trait Id { type ID: typenum::Unsigned; }
///
/// trait Different<B:Id> {}
/// impl<A:Id, B:Id> Different<B> for A
///     where A::ID: IsEqual<B::ID, Output=False> {}
///
/// struct T1;
/// impl Id for T1 { type ID = uuid_new_v4!(); }
///
/// struct T2;
/// impl Id for T2 { type ID = uuid_new_v4!(); }
///
/// fn must_be_different<A:Id, B:Id + Different<A>>(a:A, b:B) {};
///
/// must_be_different(T1, T2);
/// // must_be_different(T1, T1);  // Compile Error
/// ```
#[proc_macro]
pub fn uuid_new_v4(args: TokenStream)->TokenStream {
    let (args, prefix) = split_off_prefix(args);
    assert!(args.is_empty(), "v4 UUIDs take no arguments");
    uuid_to_tokenstream(Uuid::new_v4(), prefix)
}

/// Construct a typenum UUID
///
/// This macro parses its argument as a UUID 
/// and returns it as a `typenum::Unsigned` type:
///
/// ```edition2018
/// # use typenum_uuid::uuid;
/// type Id = uuid!(a65ff38d-b5b2-48d0-b03a-bdf468523d2e);
/// ```
#[proc_macro]
pub fn uuid(args: TokenStream)->TokenStream {
    let (args, prefix) = split_off_prefix(args);
    let args:String = args.to_string()
        .chars().filter(|c| !c.is_whitespace()).collect();
    uuid_to_tokenstream(Uuid::parse_str(&*args).unwrap(), prefix)
}
