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
        const #fn_name: opal::runtime::TypedNativeFun = {
            #(
                struct #ty_param<'m, 's>(pub opal::value::Value<'m, 's>);

                impl<'m, 's> opal::value::ValueConv<'m, 's> for #ty_param<'m, 's> {
                    const TYPE: opal::ty::BorrowedType<'static> = opal::ty::BorrowedType::Generic(#ty_param_str);
                    fn into_value(self) -> opal::value::Value<'m, 's> {
                        self.0
                    }
                    fn from_value(value: opal::value::Value<'m, 's>) -> Self {
                        #ty_param(value)
                    }
                }
            )*

            fn inner<'m, 's, 'h>(value_stack: &opal::heap::stack::Stack<'h, 's>, mutator: &'m opal::heap::mutator::Mutator<'h>, value_stack_frame: usize) -> Result<opal::value::Value<'m, 's>, RuntimeError> {
                #(
                    let #pat = <#ty as opal::value::ValueConv<'m, 's>>::from_value(value_stack.get(value_stack_frame + #index, mutator));
                )*
                <#ret_ty as opal::value::NativeFunResult<'m, 's>>::map(#block)
            }
            opal::runtime::TypedNativeFun {
                name: #fn_name_str,
                generics: &[#(#ty_param_str),*],
                params: &[#(<#ty as opal::value::ValueConv>::TYPE),*],
                returns: <<#ret_ty as opal::value::NativeFunResult>::Output as opal::value::ValueConv>::TYPE,
                fun: inner,
            }
        };
    };

    output.into()
}
