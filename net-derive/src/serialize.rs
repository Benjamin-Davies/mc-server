use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Error, Expr, ExprLit, Field, Fields, Generics, Ident,
    Lit, Result, Type, Variant, spanned::Spanned,
};

use crate::{is_primitive, parse_attrs};

pub fn expand(input: &DeriveInput) -> Result<TokenStream> {
    for attr in parse_attrs(&input.attrs) {
        let (ident, _value) = attr?;
        return Err(Error::new(ident.span(), "Unknown packet attribute"));
    }

    let ident = &input.ident;
    let generics = &input.generics;
    if generics.where_clause.is_some() {
        return Err(Error::new(
            generics.where_clause.as_ref().unwrap().span(),
            "Where clauses are not supported",
        ));
    }

    match &input.data {
        Data::Enum(data_enum) => expand_enum(ident, generics, data_enum),
        Data::Struct(data_struct) => expand_struct(ident, generics, data_struct),
        Data::Union(data_union) => Err(Error::new(
            data_union.union_token.span(),
            "Unions may not derive Packet",
        )),
    }
}

fn expand_enum(ident: &Ident, generics: &Generics, data_enum: &DataEnum) -> Result<TokenStream> {
    let variants = data_enum
        .variants
        .iter()
        .map(|variant| expand_variant(ident, variant))
        .collect::<Result<Vec<_>>>()?;

    let module = Ident::new(&format!("_serialize_{ident}"), Span::call_site());

    Ok(quote! {
        mod #module {
            use crate::{
                nbt,
                packets::serialize::{Serialize, Serializer},
            };
            use super::#ident;

            impl #generics Serialize for #ident #generics {
                fn serialize(&self, s: &mut Serializer) {
                    match self {
                        #(#variants)*
                    }
                }
            }
        }
    })
}

fn expand_struct(
    ident: &Ident,
    generics: &Generics,
    data_struct: &DataStruct,
) -> Result<TokenStream> {
    let field_names = data_struct.fields.iter().map(|field| &field.ident);

    let fields = data_struct
        .fields
        .iter()
        .map(|field| expand_field(field))
        .collect::<Result<Vec<_>>>()?;

    let module = Ident::new(&format!("_serialize_{ident}"), Span::call_site());

    Ok(quote! {
        mod #module {
            use crate::{
                nbt,
                packets::serialize::{Serialize, Serializer},
            };
            use super::#ident;

            impl #generics Serialize for #ident #generics {
                fn serialize(&self, s: &mut Serializer) {
                    let #ident { #(#field_names),* } = self;
                    #(#fields)*
                }
            }
        }
    })
}

fn expand_variant(ident: &Ident, variant: &Variant) -> Result<TokenStream> {
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
            let field_names = fields_named.named.iter().map(|field| &field.ident);
            let fields = fields_named
                .named
                .iter()
                .map(|field| expand_field(field))
                .collect::<Result<Vec<_>>>()?;
            Ok(quote_spanned! { variant.span() =>
                #ident::#variant_ident { #(#field_names),* } => {
                    s.serialize_varint(#packet_id);
                    #(#fields)*
                }
            })
        }
        Fields::Unnamed(fields_unnamed) => Err(Error::new(
            fields_unnamed.span(),
            "Packet variants with unnamed fields are not supported",
        )),
        Fields::Unit => Ok(quote_spanned! { variant.span() =>
            #ident::#variant_ident => {
                s.serialize_varint(#packet_id);
            }
        }),
    }
}

fn expand_field(field: &Field) -> Result<TokenStream> {
    let mut serialize_with = None;
    for attr in parse_attrs(&field.attrs) {
        let (ident, expr) = attr?;
        match ident.to_string().as_str() {
            "serialize_with" => {
                serialize_with = Some(expr);
            }
            _ => {
                return Err(Error::new(ident.span(), "Unknown packet attribute"));
            }
        }
    }

    let field_ident = &field.ident;

    if let Some(serialize_with) = serialize_with {
        return Ok(quote_spanned! { serialize_with.span() =>
            #serialize_with;
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
        let serialize_method = format_ident!("serialize_{field_ty_ident}");

        if is_primitive(field_ty_ident) {
            Ok(quote_spanned! { field.span() =>
                s.#serialize_method(*#field_ident);
            })
        } else {
            Ok(quote_spanned! { field.span() =>
                s.#serialize_method(#field_ident);
            })
        }
    } else {
        Ok(quote_spanned! { field.span() =>
            <super::#field_ty as Serialize>::serialize(#field_ident, s);
        })
    }
}
