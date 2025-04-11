use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::visit_mut::{self, VisitMut};
use syn::{Expr, Pat};

#[proc_macro_attribute]
pub fn sorted(_args: TokenStream, input: TokenStream) -> TokenStream {
    // eprintln!("{:#?}", input);
    let item = syn::parse(input).unwrap();

    match impl_sorted(&item) {
        Ok(ts) => ts,
        Err(err) => {
            let mut ts: TokenStream = quote! {#item}.into();
            let err_ts: TokenStream = err.to_compile_error().into();
            ts.extend(err_ts);
            ts
        }
    }
}

fn impl_sorted(item: &syn::Item) -> Result<TokenStream, syn::Error> {
    if let syn::Item::Enum(e) = item {
        let variants: Vec<_> = e.variants.iter().map(|v| v.ident.to_string()).collect();
        if let Some((i, j)) = check_sorted(&variants) {
            return Err(syn::Error::new(
                e.variants.iter().nth(j).span(),
                format!("{} should sort before {}", variants[j], variants[i],),
            ));
        }
        Ok(quote! {#item}.into())
    } else {
        Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected enum or match expression",
        ))
    }
}

#[proc_macro_attribute]
pub fn check(_args: TokenStream, input: TokenStream) -> TokenStream {
    //eprintln!("{:#?}", input);
    let mut item = syn::parse(input).unwrap();

    match impl_check(&mut item) {
        Ok(ts) => ts,
        Err(err) => {
            let mut ts: TokenStream = quote! {#item}.into();
            let err_ts: TokenStream = err.to_compile_error().into();
            ts.extend(err_ts);
            ts
        }
    }
}

#[derive(Default)]
struct MatchSortCheck {
    err: Option<syn::Error>,
}

impl VisitMut for MatchSortCheck {
    fn visit_expr_mut(&mut self, node: &mut Expr) {
        if let Expr::Match(expr) = node {
            // Using pop to strip away the #[sorted] attribute
            if let Some(attr) = expr.attrs.pop() {
                if attr.path().is_ident("sorted") {
                    //eprintln!("{:#?}", expr);
                    let arms: Vec<_> = expr
                        .arms
                        .iter()
                        .filter(|&v| match v.pat {
                            Pat::TupleStruct(_) => true,
                            _ => false,
                        })
                        .map(|v| {
                            if let Pat::TupleStruct(ts) = &v.pat {
                                ts.path.get_ident().unwrap().to_string()
                            } else {
                                unreachable!("filtered attribute must be of Pat::TupleStruct");
                            }
                        })
                        .collect();

                    // Record the error in the MatchSortCheck
                    if let Some((i, j)) = check_sorted(&arms) {
                        self.err = Some(syn::Error::new(
                            expr.arms.iter().nth(j).span(),
                            format!("{} should sort before {}", arms[j], arms[i],),
                        ));
                        return;
                    }
                }
            }
        }
        // Delegate to the default impl to visit nested expressions.
        visit_mut::visit_expr_mut(self, node);
    }
}

fn impl_check(mut item: &mut syn::ItemFn) -> Result<TokenStream, syn::Error> {
    let mut checker = MatchSortCheck::default();
    checker.visit_item_fn_mut(&mut item);

    if let Some(err) = checker.err {
        Err(err)
    } else {
        Ok(quote! {#item}.into())
    }
}

fn check_sorted(vs: &Vec<String>) -> Option<(usize, usize)> {
    for i in 0..vs.len() - 1 {
        for j in (i + 1)..vs.len() {
            let vi = &vs[i];
            let vj = &vs[j];
            if vi >= vj {
                return Some((i, j));
            }
        }
    }
    None
}
