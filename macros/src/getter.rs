use std::str::FromStr as _;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{punctuated::Punctuated, DeriveInput, Expr, Visibility};

use crate::types::CommaPunctuatedNameValues;
use crate::utils::{
    expr_to_string, retrieve_field_attrs_by_names, retrieve_struct_fields, FieldAttrPair,
};

pub(crate) fn impl_getter(input: DeriveInput) -> syn::Result<TokenStream2> {
    // 構造体の識別子を取得
    let ident = &input.ident;
    // 構造体のジェネリックスを取得
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // 構造体のフィールドのゲッターメソッドを導出
    let getter_methods = derive_getter_methods(&input)?;

    // ゲッターメソッドを構造体に実装する構文木を生成
    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #getter_methods
        }
    })
}

fn derive_getter_methods(input: &DeriveInput) -> syn::Result<TokenStream2> {
    // 構造体の可視性を取得
    let vis = &input.vis;
    // 構造体のフィールドを取得
    let fields = retrieve_struct_fields(input)?;
    // getter属性が付与されたフィールドとgetter属性を取得
    let field_attrs = retrieve_field_attrs_by_names(&fields, &["getter"]);
    // 構造体のフィールドのゲッターメソッドの構文木を生成
    let mut getter_methods: Vec<TokenStream2> = vec![];
    for field_attr in field_attrs.iter() {
        getter_methods.push(derive_getter_method(vis, field_attr)?);
    }

    // 構造体の各フィールドのゲッターメソッドの構文木を結合
    Ok(quote! {
        #(
            #getter_methods
        )*
    })
}

fn derive_getter_method(vis: &Visibility, field_attr: &FieldAttrPair) -> syn::Result<TokenStream2> {
    // フィールドの識別子を取得
    let field_ty = &field_attr.field.ty;
    let field_ident = field_attr.field.ident.as_ref().unwrap();
    // getter属性のret属性とrty属性を取得
    let values = retrieve_getter_attr_values(field_attr)?;

    let token_stream = if values.ret == "val" {
        quote! {
            #vis fn #field_ident(&self) -> #field_ty {
                self.#field_ident
            }
        }
    } else if values.ret == "ref" && values.rty.is_none() {
        quote! {
            #vis fn #field_ident(&self) -> &#field_ty {
                &self.#field_ident
            }
        }
    } else {
        let rty = TokenStream2::from_str(&values.rty.unwrap()).unwrap();
        quote! {
            #vis fn #field_ident(&self) -> #rty {
                &self.#field_ident
            }
        }
    };

    Ok(token_stream)
}

struct GetterAttrValues {
    /// getter属性のret属性の値
    pub ret: String,
    /// getter属性のrty属性の値
    pub rty: Option<String>,
}

// getter属性のカンマで区切られた属性を取得
// getter(ret = "ref", rty = "&str")
//        ^^^^^^^^^^^^^^^^^^^^^^^^^  <- この部分を取得
fn retrieve_getter_attr_values(field_attr: &FieldAttrPair) -> syn::Result<GetterAttrValues> {
    let name_values: CommaPunctuatedNameValues = field_attr
        .attr
        .parse_args_with(Punctuated::parse_terminated)
        .map_err(|err| {
            syn::Error::new_spanned(
                field_attr.attr,
                format!("failed to parse getter attribute: {}", err),
            )
        })?;

    // getterのret属性とrty属性を取得
    let mut ret: Option<Expr> = None;
    let mut rty: Option<Expr> = None;
    for nv in name_values {
        let att_ident = nv.path.get_ident().unwrap();
        if *att_ident == "ret" {
            ret = Some(nv.value);
        } else if *att_ident == "rty" {
            rty = Some(nv.value);
        }
    }

    // ret属性が指定されていない場合はエラー
    if ret.is_none() {
        return Err(syn::Error::new_spanned(
            field_attr.field,
            "getter attribute must have `ret` argument",
        ));
    }

    // ret属性の値を文字列で取得
    let ret = expr_to_string(ret).unwrap();
    // rty属性の値を文字列で取得
    let rty = expr_to_string(rty);

    // ret属性がvalの場合にrty属性が指定されていたらエラー
    if ret == "val" && rty.is_some() {
        return Err(syn::Error::new_spanned(
            field_attr.field,
            "getter attribute `rty` argument must not be specified when `ret` argument is `val`",
        ));
    }

    Ok(GetterAttrValues { ret, rty })
}
