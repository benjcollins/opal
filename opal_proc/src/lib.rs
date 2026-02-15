use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn};

#[proc_macro_attribute]
pub fn fun(_: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = syn::parse2::<ItemFn>(item.into()).expect("expected a function");

    let fn_name = item_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    let mut pats = vec![];
    let mut convs = vec![];
    let mut param_types = vec![];

    for fn_arg in &item_fn.sig.inputs {
        let FnArg::Typed(pat_type) = fn_arg else { panic!() };
        let syn::Type::Path(path) = pat_type.ty.as_ref() else {
            panic!();
        };
        let (conv, param_type) = match path.path.get_ident().expect("expected an identifier").to_string().as_str() {
            "i64" => (format_ident!("as_int"), format_ident!("Int")),
            "f64" => (format_ident!("as_float"), format_ident!("Float")),
            "bool" => (format_ident!("as_bool"), format_ident!("Bool")),
            "unit" => (format_ident!("as_unit"), format_ident!("Unit")),
            _ => panic!(),
        };
        convs.push(conv);
        pats.push(pat_type.pat.clone());
        param_types.push(param_type);
    }

    let (conv, return_type) = match item_fn.sig.output {
        syn::ReturnType::Default => (format_ident!("from_unit"), format_ident!("Unit")),
        syn::ReturnType::Type(_, ty) => {
            let syn::Type::Path(path) = ty.as_ref() else {
                panic!();
            };
            match path.path.get_ident().expect("expected an identifier").to_string().as_str() {
                "i64" => (format_ident!("from_int"), format_ident!("Int")),
                "f64" => (format_ident!("from_float"), format_ident!("Float")),
                "bool" => (format_ident!("from_bool"), format_ident!("Bool")),
                "unit" => (format_ident!("from_unit"), format_ident!("Unit")),
                _ => panic!(),
            }
        }
    };

    let index = 0..pats.len();
    let block = item_fn.block;

    let output = quote! {
        #[allow(non_upper_case_globals)]
        const #fn_name: crate::runtime::NativeFun = {
            use crate::vm::Value;
            use crate::infer::Type;
            use crate::runtime::NativeFun;

            fn inner<'f>(args: &[Value<'f>]) -> Value<'f> {
                #(
                    let #pats = args[#index].#convs();
                )*
                Value::#conv(#block)
            }
            NativeFun {
                name: #fn_name_str,
                params: &[#(Type::#param_types),*],
                returns: Type::#return_type,
                fun: inner,
            }
        };
    };

    output.into()
}
