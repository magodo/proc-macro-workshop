use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    eprintln!("{:#?}", input);
    let _ = args;
    let _ = input;

    let ast = syn::parse(input).unwrap();
    impl_sorted(&ast)
}

fn impl_sorted(ast: &syn::Item) -> TokenStream {
    _ = ast;
    quote! {/**/}.into()
}
