use std::collections::HashSet;

use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    GenericArgument, Ident, Meta, Path, PathArguments, Token, Type, TypeParamBound,
};

pub(crate) struct TypeWithPunctuatedMeta {
    pub(crate) ty:   Type,
    pub(crate) list: Punctuated<Meta, Token![,]>,
}

impl Parse for TypeWithPunctuatedMeta {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty = input.parse::<Type>()?;

        if input.is_empty() {
            return Ok(Self {
                ty,
                list: Punctuated::new(),
            });
        }

        input.parse::<Token![,]>()?;

        let list = input.parse_terminated(Meta::parse, Token![,])?;

        Ok(Self {
            ty,
            list,
        })
    }
}

/// recursive (dereference, de_ptr, de_param)
#[inline]
pub(crate) fn find_idents_in_path<'a>(
    set: &mut HashSet<&'a Ident>,
    path: &'a Path,
    recursive: Option<(bool, bool, bool)>,
) {
    if let Some((_, _, de_param)) = recursive {
        if de_param {
            if let Some(segment) = path.segments.iter().last() {
                if let PathArguments::AngleBracketed(a) = &segment.arguments {
                    // the ident is definitely not a generic parameter, so we don't insert it

                    for arg in a.args.iter() {
                        match arg {
                            GenericArgument::Type(ty) => {
                                find_idents_in_type(set, ty, recursive);
                            },
                            GenericArgument::AssocType(ty) => {
                                find_idents_in_type(set, &ty.ty, recursive);
                            },
                            _ => (),
                        }
                    }

                    return;
                }
            }
        }
    }

    if let Some(ty) = path.get_ident() {
        set.insert(ty);
    }
}

/// recursive (dereference, de_ptr, de_param)
#[inline]
pub(crate) fn find_idents_in_type<'a>(
    set: &mut HashSet<&'a Ident>,
    ty: &'a Type,
    recursive: Option<(bool, bool, bool)>,
) {
    match ty {
        Type::Array(ty) => {
            if recursive.is_some() {
                find_idents_in_type(set, ty.elem.as_ref(), recursive);
            }
        },
        Type::Group(ty) => {
            if recursive.is_some() {
                find_idents_in_type(set, ty.elem.as_ref(), recursive);
            }
        },
        Type::ImplTrait(ty) => {
            // always recursive
            for b in &ty.bounds {
                if let TypeParamBound::Trait(ty) = b {
                    find_idents_in_path(set, &ty.path, recursive);
                }
            }
        },
        Type::Macro(ty) => {
            if recursive.is_some() {
                find_idents_in_path(set, &ty.mac.path, recursive);
            }
        },
        Type::Paren(ty) => {
            if recursive.is_some() {
                find_idents_in_type(set, ty.elem.as_ref(), recursive);
            }
        },
        Type::Path(ty) => {
            find_idents_in_path(set, &ty.path, recursive);
        },
        Type::Ptr(ty) => {
            if let Some((_, true, _)) = recursive {
                find_idents_in_type(set, ty.elem.as_ref(), recursive);
            }
        },
        Type::Reference(ty) => {
            if let Some((true, ..)) = recursive {
                find_idents_in_type(set, ty.elem.as_ref(), recursive);
            }
        },
        Type::Slice(ty) => {
            if recursive.is_some() {
                find_idents_in_type(set, ty.elem.as_ref(), recursive);
            }
        },
        Type::TraitObject(ty) => {
            // always recursive
            for b in &ty.bounds {
                if let TypeParamBound::Trait(ty) = b {
                    find_idents_in_path(set, &ty.path, recursive);
                }
            }
        },
        Type::Tuple(ty) => {
            if recursive.is_some() {
                for ty in &ty.elems {
                    find_idents_in_type(set, ty, recursive)
                }
            }
        },
        _ => (),
    }
}

#[inline]
pub(crate) fn dereference(ty: &Type) -> &Type {
    if let Type::Reference(ty) = ty {
        dereference(ty.elem.as_ref())
    } else {
        ty
    }
}

#[inline]
pub(crate) fn dereference_changed(ty: &Type) -> (&Type, bool) {
    if let Type::Reference(ty) = ty {
        (dereference(ty.elem.as_ref()), true)
    } else {
        (ty, false)
    }
}
