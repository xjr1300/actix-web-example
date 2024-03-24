use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod value_field;
use value_field::{impl_value_display, impl_value_getter};
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
#[proc_macro_derive(ValueGetter, attributes(value_getter))]
pub fn derive_value_getter(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_value_getter(input) {
        Ok(token_stream) => TokenStream::from(token_stream),
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}

/// `ValueDisplay`導出マクロ
///
/// `value`フィールドを持つ構造体に、`Display`トレイトを実装する。
#[proc_macro_derive(ValueDisplay)]
pub fn derive_primitive_display(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_value_display(input) {
        Ok(token_stream) => TokenStream::from(token_stream),
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}
