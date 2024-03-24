use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod display;
use display::impl_value_display;

/// `ValueDisplay`導出マクロ
///
/// `value`フィールドを1つだけ持つ構造体に、`Display`トレイトを実装するマクロである。
#[proc_macro_derive(ValueDisplay)]
pub fn derive_primitive_display(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_value_display(input) {
        Ok(token_stream) => TokenStream::from(token_stream),
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}
