use syn::{
    Attribute, DeriveInput, Error, Expr, ExprAssign, Ident, Result, parse_macro_input,
    spanned::Spanned,
};

mod deserialize;
mod serialize;

#[proc_macro_derive(Deserialize, attributes(packet))]
pub fn derive_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    deserialize::expand(&input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Serialize, attributes(packet))]
pub fn derive_serialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    serialize::expand(&input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn parse_attrs(attrs: &[Attribute]) -> impl Iterator<Item = Result<(Ident, Expr)>> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("packet"))
        .map(|attr| {
            let args = attr.parse_args::<ExprAssign>()?;
            let Expr::Path(left) = args.left.as_ref() else {
                return Err(Error::new(args.left.span(), "Expected path"));
            };
            let left = left
                .path
                .get_ident()
                .ok_or_else(|| Error::new(left.span(), "Expected identifier"))?;
            Ok((left.clone(), *args.right))
        })
}

fn is_primitive(ident: &Ident) -> bool {
    matches!(
        ident.to_string().as_str(),
        "boolean"
            | "byte"
            | "ubyte"
            | "short"
            | "ushort"
            | "int"
            | "uint"
            | "long"
            | "ulong"
            | "float"
            | "double"
            | "varint"
            | "varlong"
            | "string"
            | "uuid"
    )
}
