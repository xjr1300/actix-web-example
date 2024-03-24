use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod domain_primitive;
use domain_primitive::impl_domain_primitive;
mod types;

/// `ValueGetter`導出マクロ
///
/// `value`フィールドを持つ構造体に、`value`メソッドを実装する。
///
/// 1. `value`フィールドが`Copy`トレイトを実装している型の場合、`#[value_getter(ret = "val")]`
/// 2. `value`フィールドが`Copy`トレイトを実装していない型で、その参照を`value`メソッドが返す場合、`#[value_getter(ret = "ref")]`
/// 3. `value`フィールドが`Copy`トレイトを実装していない型で、その型と異なる参照を`value`メソッドが返す場合、`#[value_getter(ret = "ref", rty = "&str")]`
///
/// 上記3は、`value`フィールドの型が`String`の場合を示す。
#[proc_macro_derive(DomainPrimitive, attributes(value_getter))]
pub fn derive_domain_primitive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_domain_primitive(input) {
        Ok(token_stream) => TokenStream::from(token_stream),
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}
