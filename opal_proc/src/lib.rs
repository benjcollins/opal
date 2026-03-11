use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, FnArg, Generics, ItemFn, Lifetime, ReturnType, parse::Parse, parse_macro_input,
};

fn get_attr<T: Parse>(name: &str, attrs: &[Attribute]) -> Option<T> {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident(&Ident::new(name, Span::call_site())))
        .map(|attr| attr.parse_args::<T>().unwrap())
}

#[proc_macro_derive(Root, attributes(gc_lifetime, root_ty_name, root_ty_generics))]
pub fn root(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let gc_lifetime =
        get_attr::<Lifetime>("gc_lifetime", &input.attrs).unwrap_or(Lifetime::new("'gc", Span::call_site()));
    let root_ty_name = get_attr::<Ident>("root_ty_name", &input.attrs).unwrap();
    let root_ty_generics_def = get_attr::<Generics>("root_ty_generics", &input.attrs).unwrap();
    let (root_impl_generics, root_ty_generics, root_where_clause) = root_ty_generics_def.split_for_impl();

    let output = quote! {

        struct #root_ty_name #root_ty_generics_def;

        impl #root_impl_generics opal::gc::Root for #root_ty_name #root_ty_generics #root_where_clause {
            type Ref<#gc_lifetime> = #name #ty_generics;
        }

        impl #impl_generics opal::gc::Rootable<#gc_lifetime> for #name #ty_generics #where_clause {
            type Root = #root_ty_name #root_ty_generics;
        }
    };

    output.into()
}

#[proc_macro_derive(Trace)]
pub fn trace(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let output = match input.data {
        Data::Struct(data_struct) => {
            let field_name = data_struct.fields.iter().map(|field| field.ident.clone());
            let field_ty = data_struct.fields.iter().map(|field| field.ty.clone());
            quote! {
                unsafe impl #impl_generics opal::gc::Trace for #name #ty_generics #where_clause {
                    fn trace(this: &Self, work_list: &mut opal::gc::WorkList) {
                        #(
                            <#field_ty as opal::gc::Trace>::trace(&this.#field_name, work_list);
                        )*
                    }
                }
            }
        }
        _ => panic!(),
    };
    output.into()
}

#[proc_macro_attribute]
pub fn fun(_: TokenStream, item: TokenStream) -> TokenStream {
    inner(item.into()).into()
}

fn inner(item: TokenStream2) -> TokenStream2 {
    let item_fn = syn::parse2::<ItemFn>(item).expect("expected a function");

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

    quote! {
        #[allow(non_upper_case_globals)]
        const #fn_name: opal::runtime::TypedNativeFun = {
            use opal::value::{Value, ValueConv, NativeFunResult};
            use opal::runtime::TypedNativeFun;
            use opal::heap::{Object, Values};
            use opal::ty::BorrowedType;

            #(
                struct #ty_param<'l>(pub Value<'l>);

                impl<'l> ValueConv<'l> for #ty_param<'l> {
                    const TYPE: BorrowedType<'static> = BorrowedType::Generic(#ty_param_str);
                    fn into_value(self) -> Value<'l> {
                        self.0
                    }
                    fn from_value(value: Value<'l>) -> Self {
                        #ty_param(value)
                    }
                }
            )*

            fn inner<'l>(value_stack: Object<'l, Values>, value_stack_frame: usize) -> Result<Value<'l>, RuntimeError> {
                #(
                    let #pat = <#ty as ValueConv<'l>>::from_value(value_stack.get(value_stack_frame + #index));
                )*
                <#ret_ty as NativeFunResult<'l>>::map(#block)
            }
            TypedNativeFun {
                name: #fn_name_str,
                generics: &[#(#ty_param_str),*],
                params: &[#(<#ty as ValueConv>::TYPE),*],
                returns: <<#ret_ty as NativeFunResult>::Output as ValueConv>::TYPE,
                fun: inner,
            }
        };
    }
}
