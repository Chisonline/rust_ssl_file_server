use proc_macro::TokenStream;
use syn::{parse_macro_input, spanned::Spanned, FnArg, ItemFn, PatType};
use quote::{quote, ToTokens};

#[proc_macro_attribute]
pub fn handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(item as ItemFn);
    let original_ident = input_fn.sig.ident.clone();
    let inner_ident = syn::Ident::new(
        &format!("__handler_macro_inner_{}", original_ident),
        original_ident.span(),
    );
    
    let asyncness = &input_fn.sig.asyncness;
    let publicness = &input_fn.vis;
    input_fn.sig.ident = inner_ident.clone();
    let args: Vec<_> = input_fn.sig.inputs.iter().cloned().collect();

    let arg_names: Vec<_> = args.iter().map(|arg| match arg {
        FnArg::Typed(PatType {pat, ..}) => {
            let tokens = pat.into_token_stream();
            tokens
        },
        FnArg::Receiver(r) => return syn::Error::new(
            r.span(),
            "#[handler] 不能用于包含self的方法"
        ).to_compile_error()
    }).collect();

    let new_fn = quote! {
        #input_fn

        #[allow(non_snake_case, dead_code)]
        #publicness #asyncness fn #original_ident(#(#args),*) -> Handler {
            Handler::new(#inner_ident(#(#arg_names),*))
        }
    };

    new_fn.into()
}