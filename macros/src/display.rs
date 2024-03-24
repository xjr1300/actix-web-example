use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident};

const MACRO_NAME: &str = "ValueDisplay";

pub(crate) fn impl_value_display(input: DeriveInput) -> syn::Result<TokenStream2> {
    let ident = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // 構造体であることを確認
    let Data::Struct(data_struct) = &input.data else {
        return Err(syn::Error::new(
            ident.span(),
            format!("{} is expected a struct", MACRO_NAME),
        ));
    };

    // 名前付きフィールドを取得して、タプル構造体、またはユニット構造体でないことを確認
    let Fields::Named(fields) = &data_struct.fields else {
        return Err(syn::Error::new(
            ident.span(),
            format!(
                "{} is expected a struct contain the `value` field",
                MACRO_NAME
            ),
        ));
    };
    // フィールドの名前を取得して、フィールドの数を確認
    let field_idents = fields
        .named
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect::<Vec<&Ident>>();
    if !field_idents.iter().any(|ident| *ident == "value") {
        return Err(syn::Error::new(
            ident.span(),
            format!(
                "{} is expected a struct contain the `value` field",
                MACRO_NAME
            ),
        ));
    }

    Ok(quote!(
        impl #impl_generics std::fmt::Display for #ident #ty_generics #where_clause{
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.value)
            }
        }
    ))
}
