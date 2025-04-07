use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_builder(&ast)
}

fn impl_builder(ast: &DeriveInput) -> TokenStream {
    //eprintln!("{:#?}", ast);
    let type_name = &ast.ident;
    let builder_type_name = format_ident!("{}Builder", type_name);

    let fields = match ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { ref named, .. }),
            ..
        }) => named,
        _ => panic!("malformed type"),
    };

    let field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    let gen = quote! {
        pub struct #builder_type_name {
            #(#field_names: Option<#field_types>,)*
        }

        impl #builder_type_name {
            #(
                fn #field_names(&mut self, #field_names: #field_types) -> &mut Self {
                    self.#field_names = Some(#field_names);
                    self
                }
            )*

                pub fn build(&mut self) -> Result<#type_name, Box<dyn std::error::Error>> {
                    Ok(#type_name{
                        #(#field_names: self.#field_names.take().unwrap_or(<#field_types>::default())),*
                    })
                }
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
