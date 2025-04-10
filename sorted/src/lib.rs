use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

#[proc_macro_attribute]
pub fn sorted(_args: TokenStream, input: TokenStream) -> TokenStream {
    eprintln!("{:#?}", input);
    let mut input_ts = input.clone();
    let item = syn::parse(input).unwrap();

    match impl_sorted(&item) {
        Ok(ts) => ts,
        Err(err) => {
            let err_ts: TokenStream = err.to_compile_error().into();
            input_ts.extend(err_ts);
            input_ts
        }
    }
}

fn impl_sorted(item: &syn::Item) -> Result<TokenStream, syn::Error> {
    if let syn::Item::Enum(e) = item {
        let variants: Vec<_> = e.variants.iter().collect();
        for i in 0..variants.len() - 1 {
            for j in (i + 1)..variants.len() {
                let ident_i = &variants[i].ident;
                let ident_j = &variants[j].ident;
                if ident_i >= ident_j {
                    return Err(syn::Error::new(
                        variants[j].span(),
                        format!("{} should sort before {}", ident_j, ident_i,),
                    ));
                }
            }
        }
        Ok(quote! {/**/}.into())
    } else {
        Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected enum or match expression",
        ))
    }
}
