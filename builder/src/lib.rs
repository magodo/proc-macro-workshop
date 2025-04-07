use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    AngleBracketedGenericArguments, Data, DataStruct, DeriveInput, Fields, FieldsNamed,
    GenericArgument, Ident, Path, PathArguments, PathSegment, Type, TypePath,
};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_builder(&ast)
}

fn option_inner_type(ty: &Type) -> Option<Ident> {
    if let Type::Path(TypePath {
        path: Path { ref segments, .. },
        ..
    }) = ty
    {
        if let Some(PathSegment { ident, arguments }) = segments.first() {
            if ident == "Option" {
                if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    args, ..
                }) = arguments
                {
                    if let Some(GenericArgument::Type(Type::Path(TypePath {
                        path: Path { ref segments, .. },
                        ..
                    }))) = args.first()
                    {
                        if let Some(PathSegment { ident, .. }) = segments.first() {
                            return Some(ident.clone());
                        }
                    }
                }
            }
        }
    }
    None
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

    let field_name_method = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;

        if let Some(ty) = option_inner_type(ty) {
            return quote! {
                fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            };
        }
        quote! {
            fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = #name;
                self
            }
        }
    });

    let field_name_build = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        quote! {
            #name: self.#name.clone(),
        }
    });

    let gen = quote! {
        pub struct #builder_type_name {
            #(#field_names: #field_types,)*
        }

        impl #builder_type_name {
            #(#field_name_method)*

                pub fn build(&mut self) -> Result<#type_name, Box<dyn std::error::Error>> {
                    Ok(#type_name{
                        #(#field_name_build)*
                    })
                }
        }

        impl #type_name {
            pub fn builder() -> #builder_type_name {
                #builder_type_name {
                    #(#field_names: <#field_types>::default()),*
                }
            }
        }
    };
    TokenStream::from(gen)
}
