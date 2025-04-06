use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_builder(&ast)
}

fn impl_builder(ast: &DeriveInput) -> TokenStream {
    eprintln!("{:#?}", ast);
    let type_name = &ast.ident;
    let builder_type_name = format_ident!("{}Builder", type_name);
    let gen = quote! {
        pub struct #builder_type_name {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }

        impl #type_name {
            pub fn builder() -> #builder_type_name {
                #builder_type_name {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }
    };
    TokenStream::from(gen)
}
