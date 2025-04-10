use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    eprintln!("{:#?}", input);
    let input = syn::parse(input).unwrap();

    match impl_sorted(&input) {
        Ok(ts) => ts,
        Err(err) => err.to_compile_error().into(),
    }
}

fn impl_sorted(input: &syn::Item) -> Result<TokenStream, syn::Error> {
    match input {
        syn::Item::Enum(e) => {
            Ok(quote! {/**/}.into())
        }
        _ => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected enum or match expression",
        )),
    }
}
