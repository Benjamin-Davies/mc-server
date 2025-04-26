use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Expr, ExprAssign, ExprLit, ExprPath,
    Field, Fields, Ident, Lit, Result, Type, Variant, parse_macro_input, spanned::Spanned,
};

#[proc_macro_derive(Deserialize, attributes(packet))]
pub fn derive_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand_deserialize(&input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand_deserialize(input: &DeriveInput) -> Result<TokenStream> {
    let mut state = None;
    for attr in parse_attrs(&input.attrs) {
        let (ident, value) = attr?;
        match ident.to_string().as_str() {
            "state" => {
                let Expr::Path(path) = value else {
                    return Err(Error::new(value.span(), "Expected path"));
                };
                state = Some(path.clone());
            }
            _ => {
                return Err(Error::new(ident.span(), "Unknown packet attribute"));
            }
        }
    }

    let ident = &input.ident;

    match &input.data {
        Data::Enum(data_enum) => {
            let state =
                state.ok_or_else(|| Error::new(input.ident.span(), "Missing `state` attribute"))?;
            expand_deserialize_enum(ident, data_enum, &state)
        }
        Data::Struct(data_struct) => expand_deserialize_struct(ident, data_struct),
        Data::Union(data_union) => Err(Error::new(
            data_union.union_token.span(),
            "Unions may not derive Packet",
        )),
    }
}

fn expand_deserialize_enum(
    ident: &Ident,
    data_enum: &DataEnum,
    state: &ExprPath,
) -> Result<TokenStream> {
    let variants = data_enum
        .variants
        .iter()
        .map(|variant| expand_deserialize_variant(ident, variant))
        .collect::<Result<Vec<_>>>()?;

    let module = Ident::new(&format!("_deserialize_{ident}"), Span::call_site());

    Ok(quote! {
        mod #module {
            use crate::{
                connection::State,
                packets::deserialize::{Deserialize, Deserializer, Error, InvalidPacketIdSnafu},
            };
            use super::#ident;

            impl<'de> Deserialize<'de> for #ident {
                fn deserialize(d: &mut Deserializer<'de>) -> Result<Self, Error> {
                    match d.deserialize_varint()? {
                        #(#variants)*
                        packet_id => Err(InvalidPacketIdSnafu { state: State::#state, packet_id }.build()),
                    }
                }
            }
        }
    })
}

fn expand_deserialize_struct(ident: &Ident, data_struct: &DataStruct) -> Result<TokenStream> {
    let fields = data_struct
        .fields
        .iter()
        .map(|field| expand_deserialize_field(field))
        .collect::<Result<Vec<_>>>()?;

    let module = Ident::new(&format!("_deserialize_{ident}"), Span::call_site());

    Ok(quote! {
        mod #module {
            use crate::{
                connection::State,
                packets::deserialize::{Deserialize, Deserializer, Error},
            };
            use super::#ident;

            impl<'de> Deserialize<'de> for #ident {
                fn deserialize(d: &mut Deserializer<'de>) -> Result<Self, Error> {
                    Ok(#ident {
                        #(#fields)*
                    })
                }
            }
        }
    })
}

fn expand_deserialize_variant(ident: &Ident, variant: &Variant) -> Result<TokenStream> {
    let mut packet_id = None;
    for attr in parse_attrs(&variant.attrs) {
        let (ident, expr) = attr?;
        match ident.to_string().as_str() {
            "id" => {
                let Expr::Lit(ExprLit {
                    lit: Lit::Int(lit), ..
                }) = expr
                else {
                    return Err(Error::new(expr.span(), "Expected integer literal"));
                };
                packet_id = Some(lit.clone());
            }
            _ => {
                return Err(Error::new(ident.span(), "Unknown packet attribute"));
            }
        }
    }
    let packet_id =
        packet_id.ok_or_else(|| Error::new(variant.ident.span(), "Missing `id` attribute"))?;

    let variant_ident = &variant.ident;

    match &variant.fields {
        Fields::Named(fields_named) => {
            let fields = fields_named
                .named
                .iter()
                .map(|field| expand_deserialize_field(field))
                .collect::<Result<Vec<_>>>()?;
            Ok(quote_spanned! { variant.span() =>
                #packet_id => Ok(#ident::#variant_ident { #(#fields)* }),
            })
        }
        Fields::Unnamed(fields_unnamed) => Err(Error::new(
            fields_unnamed.span(),
            "Packet variants with unnamed fields are not supported",
        )),
        Fields::Unit => Ok(quote_spanned! { variant.span() =>
            #packet_id => Ok(#ident::#variant_ident),
        }),
    }
}

fn expand_deserialize_field(field: &Field) -> Result<TokenStream> {
    let mut deserialize_with = None;
    for attr in parse_attrs(&field.attrs) {
        let (ident, expr) = attr?;
        match ident.to_string().as_str() {
            "deserialize_with" => {
                deserialize_with = Some(expr);
            }
            _ => {
                return Err(Error::new(ident.span(), "Unknown packet attribute"));
            }
        }
    }

    let field_ident = &field.ident;

    if let Some(deserialize_with) = deserialize_with {
        return Ok(quote_spanned! { deserialize_with.span() =>
            #field_ident: #deserialize_with,
        });
    }

    let Type::Path(field_ty) = &field.ty else {
        return Err(Error::new(field.ty.span(), "Expected path type"));
    };
    let field_ty_ident = &field_ty.path.segments.last().unwrap().ident;

    if field_ty_ident
        .to_string()
        .chars()
        .next()
        .is_some_and(char::is_lowercase)
    {
        let deserialize_method = format_ident!("deserialize_{field_ty_ident}");

        Ok(quote_spanned! { field.span() =>
            #field_ident: d.#deserialize_method()?,
        })
    } else {
        Ok(quote_spanned! { field.span() =>
            #field_ident: super::#field_ty::deserialize(d)?,
        })
    }
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
