use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, FnArg, GenericParam, ItemFn, Lifetime, ReturnType, Type, parse::Parse,
    parse_macro_input, parse_quote,
};

fn get_attr<T: Parse>(name: &str, attrs: &[Attribute]) -> Option<T> {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident(&Ident::new(name, Span::call_site())))
        .map(|attr| attr.parse_args::<T>().unwrap())
}

#[proc_macro_derive(Rootable, attributes(lifetime, root, clause))]
pub fn root(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let gc_lifetime = get_attr::<Lifetime>("lifetime", &input.attrs).unwrap_or(Lifetime::new("'gc", Span::call_site()));
    let root_ty = get_attr::<Type>("root", &input.attrs).unwrap();

    let output = quote! {
        impl #impl_generics opal::gc::Rootable for #name #ty_generics #where_clause {
            type Root<#gc_lifetime> = #root_ty;
        }
    };

    output.into()
}

#[proc_macro_derive(Trace)]
pub fn trace(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    for param in &mut input.generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(opal::gc::Trace));
        }
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let output = match input.data {
        Data::Struct(data_struct) => {
            let member = data_struct.fields.members();
            let field_ty = data_struct.fields.iter().map(|field| field.ty.clone());
            quote! {
                unsafe impl #impl_generics opal::gc::Trace for #name #ty_generics #where_clause {
                    const TRACE: bool = #(<#field_ty as opal::gc::Trace>::TRACE)||*;

                    fn trace(this: &Self, work_list: &mut opal::gc::WorkList) {
                        #(
                            opal::gc::Trace::trace(&this.#member, work_list);
                        )*
                    }
                }
            }
        }
        Data::Enum(data_enum) => {
            let ty = data_enum
                .variants
                .iter()
                .flat_map(|variant| variant.fields.iter().map(|field| field.ty.clone()));

            let match_variant = data_enum.variants.iter().map(|variant| {
                let variant_name = variant.ident.clone();
                let member = variant.fields.members();
                let var_name: Vec<_> = (0..variant.fields.len()).map(|id| format_ident!("_{}", id)).collect();
                quote! {
                    #name::#variant_name { #(#member: #var_name),* } => {
                        #(
                            opal::gc::Trace::trace(#var_name, work_list);
                        )*
                    }
                }
            });

            quote! {
                unsafe impl #impl_generics opal::gc::Trace for #name #ty_generics #where_clause {
                    const TRACE: bool = #(<#ty as opal::gc::Trace>::TRACE)||*;

                    fn trace(this: &Self, work_list: &mut opal::gc::WorkList) {
                        match this {
                            #(#match_variant)*
                        }
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

            fn inner<'m, 's>(value_stack: opal::heap::Object<'m, opal::heap::Array<'s>>, value_stack_frame: usize) -> Result<opal::value::Value<'m, 's>, RuntimeError> {
                #(
                    let #pat = <#ty as opal::value::ValueConv<'m, 's>>::from_value(value_stack.get(value_stack_frame + #index));
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
    }
}
