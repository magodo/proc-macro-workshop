use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DataStruct, DeriveInput, Expr, ExprAssign, ExprLit, Fields, FieldsNamed,
    GenericArgument, Ident, Lit, Path, PathArguments, PathSegment, Type, TypePath,
};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_builder(&ast)
}

fn inner_type<'a>(wrapper: &str, ty: &'a Type) -> Option<&'a Type> {
    if let Type::Path(TypePath {
        path: Path { ref segments, .. },
        ..
    }) = ty
    {
        if let Some(PathSegment { ident, arguments }) = segments.first() {
            if ident == wrapper {
                if let PathArguments::AngleBracketed(arg) = arguments {
                    if let Some(GenericArgument::Type(typ)) = arg.args.first() {
                        return Some(typ);
                    }
                }
            }
        }
    }
    None
}

#[derive(Default)]
struct FieldAttribute {
    each_name: Option<Ident>,
}

fn attr_builder_value(attrs: &Vec<Attribute>) -> Option<FieldAttribute> {
    if attrs.len() == 0 {
        return None;
    }
    let mut field_attribute = FieldAttribute::default();
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("builder"))
        .for_each(|attr| {
            if let Ok(Expr::Assign(ExprAssign { left, right, .. })) = attr.parse_args() {
                if let Expr::Path(expr) = *left {
                    if let Some(ident) = expr.path.get_ident() {
                        if ident == "each" {
                            if let Expr::Lit(ExprLit {
                                lit: Lit::Str(lit_str),
                                ..
                            }) = *right
                            {
                                field_attribute.each_name =
                                    Some(Ident::new(&lit_str.value(), Span::call_site()));
                            }
                        }
                    }
                }
            }
        });
    Some(field_attribute)
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

        if let Some(ty) = inner_type("Option", ty) {
            return quote! {
                fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            };
        } else if let Some(attr) = attr_builder_value(&f.attrs) {
            if let Some(each_name) = attr.each_name {
                let ty = inner_type("Vec", ty).unwrap();
                return quote! {
                    fn #each_name(&mut self, #each_name: #ty) -> &mut Self {
                        self.#name.push(#each_name);
                        self
                    }
                };
            }
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
