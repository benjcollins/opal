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
        const #fn_name: opal::runtime::TypedNativeFun = {
            use opal::value::{Value, ValueConv, NativeFunResult};
            use opal::runtime::TypedNativeFun;
            use opal::heap::{Object, Values};

            fn inner<'l>(value_stack: Object<'l, Values>, value_stack_frame: usize) -> Result<Value<'l>, RuntimeError> {
                #(
                    let #pat = <#ty as ValueConv<'l>>::from_value(value_stack.get(value_stack_frame + #index));
                )*
                <#ret_ty as NativeFunResult<'l>>::map(#block)
            }
            TypedNativeFun {
                name: #fn_name_str,
                params: &[#(<#ty as ValueConv>::TYPE),*],
                returns: <<#ret_ty as NativeFunResult>::Output as ValueConv>::TYPE,
                fun: inner,
            }
        };
    };

    output
}
