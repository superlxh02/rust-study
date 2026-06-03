use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    DeriveInput,
    Ident,
    ItemFn,
    LitStr,
    Token,
};

// ===============================
// 1. 函数式过程宏
// make_fn!(hello, "world");
// ===============================

struct MakeFnInput {
    name: Ident,
    _comma: Token![,],
    message: LitStr,
}

impl Parse for MakeFnInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            _comma: input.parse()?,
            message: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn make_fn(input: TokenStream) -> TokenStream {
    let MakeFnInput { name, message, .. } =
        parse_macro_input!(input as MakeFnInput);

    quote! {
        fn #name() {
            println!("{}", #message);
        }
    }
        .into()
}

// ===============================
// 2. derive 过程宏
// #[derive(HelloDerive)]
// ===============================

#[proc_macro_derive(HelloDerive)]
pub fn hello_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = input.generics;

    let (impl_generics, ty_generics, where_clause) =
        generics.split_for_impl();

    quote! {
        impl #impl_generics Hello for #name #ty_generics #where_clause {
            fn hello(&self) {
                println!("Hello from {}", stringify!(#name));
            }
        }
    }
        .into()
}

// ===============================
// 3. 属性过程宏
// #[log_call]
// ===============================

#[proc_macro_attribute]
pub fn log_call(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let attrs = input_fn.attrs;
    let vis = input_fn.vis;
    let sig = input_fn.sig;
    let block = input_fn.block;

    let fn_name = &sig.ident;

    quote! {
        #(#attrs)*
        #vis #sig {
            println!("calling {}", stringify!(#fn_name));
            #block
        }
    }
        .into()
}