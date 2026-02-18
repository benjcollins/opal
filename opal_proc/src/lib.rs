use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{FnArg, ItemFn, ReturnType};

#[proc_macro_attribute]
pub fn fun(_: TokenStream, item: TokenStream) -> TokenStream {
    inner(item.into()).into()
}

fn inner(item: TokenStream2) -> TokenStream2 {
    let item_fn = syn::parse2::<ItemFn>(item).expect("expected a function");

    let fn_name = item_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    let mut pat = vec![];
    let mut ty = vec![];

    for fn_arg in &item_fn.sig.inputs {
        let FnArg::Typed(pat_type) = fn_arg else { panic!() };
        pat.push(pat_type.pat.clone());
        ty.push(pat_type.ty.clone());
    }

    let ReturnType::Type(_, ret_ty) = item_fn.sig.output.clone() else {
        panic!()
    };

    let index = 0..pat.len();
    let block = item_fn.block;

    let output = quote! {
        #[allow(non_upper_case_globals)]
        const #fn_name: opal::runtime::NativeFun = {
            use opal::vm::{Value, ValueConv, NativeFunResult};
            use opal::infer::Type;
            use opal::runtime::NativeFun;

            fn inner<'f>(args: &[Value<'f>]) -> Result<Value<'f>, RuntimeError> {
                #(
                    let #pat = <#ty as ValueConv>::from_value(args[#index]);
                )*
                <#ret_ty as NativeFunResult>::map(#block)
            }
            NativeFun {
                name: #fn_name_str,
                params: &[#(<#ty as ValueConv>::TYPE),*],
                returns: <<#ret_ty as NativeFunResult>::Output as ValueConv>::TYPE,
                fun: inner,
            }
        };
    };

    output
}
