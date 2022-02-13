use quote::quote;
use syn::{
    GenericArgument, PathArguments, ReturnType, TraitBound, Type, TypeParamBound, TypePath,
    TypeTraitObject,
};

const PIN: &str = "Pin";
const BOX: &str = "Box";
const FUTURE: &str = "Future";
const RESULT: &str = "Result";

pub struct PinBoxFutRet {
    pub is_pin_box_fut:    bool,
    pub is_fut_ret_result: bool,
    pub ret_ty:            proc_macro2::TokenStream,
}

impl Default for PinBoxFutRet {
    fn default() -> Self {
        PinBoxFutRet {
            is_pin_box_fut:    false,
            is_fut_ret_result: false,
            ret_ty:            quote! {},
        }
    }
}

impl PinBoxFutRet {
    pub fn parse(ret_ty: &ReturnType) -> PinBoxFutRet {
        let expect_ty = match ret_ty {
            ReturnType::Type(_, ty) => ty,
            _ => return PinBoxFutRet::default(),
        };

        let expect_pin = match *(expect_ty.clone()) {
            Type::Path(TypePath { qself: _, path }) => {
                let last_seg = path.segments.last().cloned();
                match last_seg.map(|ls| (ls.ident.clone(), ls)) {
                    Some((ls_ident, ls)) if ls_ident == PIN => ls,
                    _ => return PinBoxFutRet::default(),
                }
            }
            _ => return PinBoxFutRet::default(),
        };

        let expect_box = match &expect_pin.arguments {
            PathArguments::AngleBracketed(wrapper) => match wrapper.args.last() {
                Some(GenericArgument::Type(Type::Path(TypePath { qself: _, path }))) => {
                    match path.segments.last().map(|ls| (ls.ident.clone(), ls)) {
                        Some((ls_ident, ls)) if ls_ident == BOX => ls,
                        _ => return PinBoxFutRet::default(),
                    }
                }
                _ => return PinBoxFutRet::default(),
            },
            _ => return PinBoxFutRet::default(),
        };

        // Has Future trait bound
        match &expect_box.arguments {
            PathArguments::AngleBracketed(wrapper) => match wrapper.args.last() {
                Some(GenericArgument::Type(Type::TraitObject(TypeTraitObject {
                    dyn_token: _,
                    bounds,
                }))) => {
                    let mut fut_ret = PinBoxFutRet::default();

                    for bound in bounds.iter() {
                        if let TypeParamBound::Trait(TraitBound { path, .. }) = bound {
                            if let Some(arg) = path.segments.last() {
                                if arg.ident == FUTURE {
                                    fut_ret.is_pin_box_fut = true;
                                    fut_ret.is_fut_ret_result =
                                        is_fut_ret_result(&arg.arguments, &mut fut_ret);
                                    break;
                                }
                            }
                        }
                    }
                    fut_ret
                }
                _ => PinBoxFutRet::default(),
            },
            _ => PinBoxFutRet::default(),
        }
    }
}

fn is_fut_ret_result(input: &PathArguments, fut_ret: &mut PinBoxFutRet) -> bool {
    match input {
        PathArguments::AngleBracketed(angle_arg) => {
            match angle_arg.args.first().expect("future output") {
                GenericArgument::Binding(binding) => match &binding.ty {
                    Type::Path(path) => {
                        fut_ret.ret_ty = quote! { #path };
                        path.path
                            .segments
                            .last()
                            .unwrap()
                            .ident
                            .to_string()
                            .contains(RESULT)
                    }
                    _ => false,
                },
                _ => false,
            }
        }
        _ => false,
    }
}
