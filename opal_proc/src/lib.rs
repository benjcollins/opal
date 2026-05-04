use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, ReturnType};

#[proc_macro_attribute]
pub fn fun(_: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = syn::parse2::<ItemFn>(item.into()).expect("expected a function");

    let fn_name = item_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    let mut ty_param = vec![];
    let mut ty_param_str = vec![];
    let mut pat = vec![];
    let mut ty = vec![];

    for x in item_fn.sig.generics.type_params() {
        ty_param.push(x.ident.clone());
        ty_param_str.push(x.ident.to_string());
    }

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
        const #fn_name: opal::runtime::TypedHostFun = {
            #(
                struct #ty_param<'h>(pub opal::value::Value<'h>);

                impl<'h> opal::value::ValueConv<'h> for #ty_param<'h> {
                    const TYPE: opal::ty::BorrowedType<'static> = opal::ty::BorrowedType::Generic(#ty_param_str);
                    fn into(self) -> opal::value::Value<'h> {
                        self.0
                    }
                    fn from(value: opal::value::Value<'h>) -> Self {
                        #ty_param(value)
                    }
                }
            )*

            fn inner<'h>(stack: &opal::heap::stack::StackGuard<'h>) -> Result<opal::value::Value<'h>, RuntimeError> {
                #(
                    let #pat = <#ty as opal::value::ValueConv<'h>>::from(stack.get_stack_value(stack.base_ptr + #index));
                )*
                <#ret_ty as opal::value::HostFunResult<'h>>::map(#block)
            }
            opal::runtime::TypedHostFun {
                name: #fn_name_str,
                generics: &[#(#ty_param_str),*],
                params: &[#(<#ty as opal::value::ValueConv>::TYPE),*],
                returns: <<#ret_ty as opal::value::HostFunResult>::Output as opal::value::ValueConv>::TYPE,
                fun: inner,
            }
        };
    };

    output.into()
}
