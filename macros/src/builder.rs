use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{
    AngleBracketedGenericArguments, Attribute, Data, DataStruct, DeriveInput, Fields, FieldsNamed,
    GenericArgument, Ident, Lit, Path, PathArguments, PathSegment, Type, TypePath, Visibility,
};

use crate::types::CommaPunctuatedFields;
use crate::utils::retrieve_name_values_list;

pub(crate) fn impl_builder(input: DeriveInput) -> syn::Result<TokenStream2> {
    if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
    }) = input.data
    {
        let struct_ident = input.ident;
        let vis = input.vis;
        let builder_ident = format_ident!("{}Builder", struct_ident);

        // ビルダーを構築する構造体のフィールドの識別子と型を取得
        let fields = retrieve_struct_field_ident_and_type_pairs(&named)?;
        // ビルダー構造体を実装
        let builder_struct = impl_builder_struct(&vis, &builder_ident, &fields);
        // ビルダーの`new`メソッドを実装
        let builder_new_method = impl_builder_new_method(&vis, &fields);
        // ビルダーを構築する構造体のフィールドの`builder`フィールドに付与された名前`each`の値を取得
        let each_values = retrieve_each_name_value(&named)?;
        // ビルダーのsetterメソッドを実装
        let builder_setter_methods = impl_builder_setter_methods(&vis, &fields, &each_values);
        // ビルダーの`build`メソッドを実装
        let func_ident = retrieve_builder_validation_func(&input.attrs)?;
        let builder_build_method =
            impl_builder_build_method(&vis, &struct_ident, &fields, func_ident);

        Ok(quote! {
            #builder_struct

            impl #builder_ident {
                #builder_new_method

                #builder_setter_methods

                #builder_build_method
            }
        })
    } else {
        Err(syn::Error::new(input.span(), "only struct supported"))
    }
}

/// ビルダーが構築するプリミティブを検証する関数の識別子を取得する。
///
/// ```text
/// #[derive(Builder)]
/// #[builder_validation(func = "func_name")
/// struct Foo { ... }
/// ```
/// 上記`func_name`を取得する。
fn retrieve_builder_validation_func(attrs: &[Attribute]) -> syn::Result<Option<Ident>> {
    let name_values_list = retrieve_name_values_list(attrs, "builder_validation")?;

    // builder_validation属性が指定されていない場合は検証しない
    if name_values_list.is_empty() {
        return Ok(None);
    }
    // builder_validation属性が2つ以上指定されている場合はエラー
    if name_values_list.len() > 1 {
        return Err(syn::Error::new(
            attrs[0].span(),
            "only one builder_validation can be specified",
        ));
    }
    // builder_validation属性にfuncのみ指定されているか確認
    let name_values = &name_values_list[0];
    if 1 < name_values.keys().len() {
        return Err(syn::Error::new(
            attrs[0].span(),
            "builder_validation must have only one `func` name value",
        ));
    }

    // builder_validation属性にfuncが複数指定されている場合はエラー
    let func_list = name_values
        .get(&format_ident!("func"))
        .ok_or(syn::Error::new(
            attrs[0].span(),
            "builder_validation must have only one `func` name value",
        ))?;
    if 1 < func_list.len() {
        return Err(syn::Error::new(
            attrs[0].span(),
            "only one `func` can be specified",
        ));
    }

    match &func_list[0] {
        Lit::Str(s) => Ok(Some(format_ident!("{}", s.value()))),
        _ => Err(syn::Error::new(
            attrs[0].span(),
            "func must have a function name string",
        )),
    }
}

/// ビルダーを構築する構造体のフィールドに付与された`builder`属性の`each`を取得する。
///
/// ```text
/// #[derive(Builder)]
/// struct Foo {
///     #[builder(each = "each_name")]
///     a: String,
/// }
/// ```
///
/// 上記`each_name`を取得する。
fn retrieve_builder_each(attrs: &[Attribute]) -> syn::Result<Option<Ident>> {
    let name_values_list = retrieve_name_values_list(attrs, "builder")?;

    // builder属性が指定されていない場合
    if name_values_list.is_empty() {
        return Ok(None);
    }
    // builder属性が2つ以上指定されている場合はエラー
    if name_values_list.len() > 1 {
        return Err(syn::Error::new(
            attrs[0].span(),
            "only one builder can be specified",
        ));
    }
    // builder属性にeachのみ指定されているか確認
    let name_values = &name_values_list[0];
    if 1 < name_values.keys().len() {
        return Err(syn::Error::new(
            attrs[0].span(),
            "builder must have only one `each` name value",
        ));
    }

    // builder属性にeachが複数指定されている場合はエラー
    let each_list = name_values
        .get(&format_ident!("each"))
        .ok_or(syn::Error::new(
            attrs[0].span(),
            "builder must have only one `each` name value",
        ))?;
    if 1 < each_list.len() {
        return Err(syn::Error::new(
            attrs[0].span(),
            "only one each can be specified",
        ));
    }

    match &each_list[0] {
        Lit::Str(s) => Ok(Some(format_ident!("{}", s.value()))),
        _ => Err(syn::Error::new(
            attrs[0].span(),
            "each must have a method name string",
        )),
    }
}

/// ビルダーを構築する構造体のフィールド情報
struct FieldInfo<'a> {
    /// フィールドの識別子
    ident: &'a Ident,
    /// フィールドの型
    ty: &'a Type,
}

/// ビルダーを構築する構造体のフィールドの識別子と型を取得する。
fn retrieve_struct_field_ident_and_type_pairs(
    named_fields: &CommaPunctuatedFields,
) -> syn::Result<Vec<FieldInfo>> {
    let mut fields = vec![];
    for named_field in named_fields {
        let ident = named_field.ident.as_ref();
        if ident.is_none() {
            return Err(syn::Error::new(
                named_field.span(),
                "field must have an ident",
            ));
        }
        fields.push(FieldInfo {
            ident: ident.unwrap(),
            ty: &named_field.ty,
        });
    }

    Ok(fields)
}

/// ビルダー構造体を実装する。
fn impl_builder_struct(
    vis: &Visibility,
    builder_ident: &Ident,
    fields: &[FieldInfo],
) -> TokenStream2 {
    let field_tokens = fields
        .iter()
        .map(|FieldInfo { ident, ty }| match field_type(ty) {
            FieldType::Option(inner_ty) => quote! { #ident: ::core::option::Option<#inner_ty> },
            _ => quote! { #ident: ::core::option::Option<#ty> },
        })
        .collect::<Vec<TokenStream2>>();

    quote! {
        #vis struct #builder_ident {
            #(#field_tokens),*
        }
    }
}

/// ビルダーの`new`メソッドを実装する。
fn impl_builder_new_method(vis: &Visibility, fields: &[FieldInfo]) -> TokenStream2 {
    let field_tokens = fields
        .iter()
        .map(|FieldInfo { ident, ty }| match field_type(ty) {
            FieldType::Vec(_) => {
                quote! { #ident: ::core::option::Option::Some(::std::vec::Vec::new()) }
            }
            FieldType::Option(_) | FieldType::Raw => {
                quote! { #ident: ::core::option::Option::None }
            }
        })
        .collect::<Vec<TokenStream2>>();

    quote! {
        #vis fn new() -> Self {
            Self {
                #(#field_tokens),*
            }
        }
    }
}

/// ビルダーを構築する構造体のフィールドに付与された`builder`フィールドの`each`の値を取得する。
///
/// # 戻り値
///
/// フィールドの`builder`属性に`each`が存在する場合は`each`の値、存在しない場合は`None`を格納したベクタ
fn retrieve_each_name_value(fields: &CommaPunctuatedFields) -> syn::Result<Vec<Option<Ident>>> {
    fields
        .iter()
        .map(|f| retrieve_builder_each(&f.attrs))
        .collect::<syn::Result<Vec<_>>>()
}

/// ビルダーのsetterメソッドを実装する。
fn impl_builder_setter_methods(
    vis: &Visibility,
    fields: &[FieldInfo],
    each_attrs: &[Option<Ident>],
) -> TokenStream2 {
    let setters =
        fields
            .iter()
            .zip(each_attrs)
            .map(|(FieldInfo { ident, ty }, maybe_each)| {
                let has_each = maybe_each.is_some();
                match field_type(ty) {
                    FieldType::Option(inner_ty) => {
                        quote! {
                            #vis fn #ident (&mut self, #ident: ::core::option::Option<#inner_ty>) -> &mut Self {
                                self.#ident = #ident;
                                self
                            }
                        }
                    }
                    FieldType::Vec(inner_ty) if has_each => {
                        let each = maybe_each.as_ref().unwrap();
                        quote! {
                            #vis fn #each (&mut self, #each: #inner_ty) -> &mut Self {
                                self.#ident.as_mut().map(|v| v.push(#each));
                                self
                            }
                        }
                    }
                    _ => {
                        quote! {
                            #vis fn #ident (&mut self, #ident: #ty) -> &mut Self {
                                self.#ident = ::core::option::Option::Some(#ident);
                                self
                            }
                        }
                    }
                }
            });

    quote! {
        #(#setters)*
    }
}

/// ビルダーの`build`メソッドを実装する。
///
/// # 引数
///
/// * `vis` - `build`メソッドの可視性
/// * `struct_ident` - ビルダーを構築する構造体の識別子
/// * `field` - ビルダーを構築する構造体のフィールド
/// * `func_ident` - ビルダーを構築する構造体を検証するメソッドの識別子
fn impl_builder_build_method(
    vis: &Visibility,
    struct_ident: &Ident,
    fields: &[FieldInfo],
    func: Option<Ident>,
) -> TokenStream2 {
    let field_tokens = fields.iter().map(|FieldInfo{ident, ty}|
    match field_type(ty) {
        FieldType::Option(_) => quote! {
            #ident: match self.#ident {
                ::core::option::Option::Some(_) => ::core::option::Option::Some(self.#ident.take().unwrap()),
                ::core::option::Option::None => ::core::option::Option::None,
            }
        },
        _ => quote! {
            #ident: self.#ident.take().ok_or_else(||
                format!("{} is not provided", stringify!(#ident))
            )?
        },
    });

    let instance = format_ident!("{}", "instance");
    let validator = match func {
        Some(func) => {
            let func_ident = format_ident!("{}", func);
            quote!(
                #func_ident(&#instance)?;
            )
        }
        None => quote!(),
    };

    quote! {
        #vis fn build(&mut self) -> ::core::result::Result<
                #struct_ident,
                ::std::boxed::Box<dyn ::std::error::Error>
            >
            {
                let #instance = #struct_ident {
                    #(#field_tokens),*
                };

                #validator

                Ok(#instance)
            }
    }
}

/// フィールドの型
enum FieldType {
    /// ラップされていない型
    Raw,

    /// `Option`型
    ///
    /// タプルの値は`Option`でラップされた型
    Option(Type),

    /// `Vec`型
    ///
    /// タプルの値は`Vec`でラップされた型
    Vec(Type),
}

/// フィールドの型を取得する。
fn field_type(ty: &Type) -> FieldType {
    if let Type::Path(TypePath {
        qself: None,
        path: Path {
            leading_colon,
            segments,
        },
    }) = ty
    {
        if leading_colon.is_none() && segments.len() == 1 {
            if let Some(PathSegment {
                ident,
                arguments:
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }),
            }) = segments.first()
            {
                if let (1, Some(GenericArgument::Type(t))) = (args.len(), args.first()) {
                    if ident == "Option" {
                        return FieldType::Option(t.clone());
                    } else if ident == "Vec" {
                        return FieldType::Vec(t.clone());
                    }
                }
            }
        }
    }

    FieldType::Raw
}
