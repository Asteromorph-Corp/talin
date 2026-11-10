use syn::visit::Visit;
use syn::visit_mut::VisitMut;

/// Checks if the given type is `str`.
pub(crate) fn is_str_type(ty: &syn::Type) -> bool {
    let syn::Type::Path(p) = ty else {
        return false;
    };

    p.path.is_ident("str")
}

/// Checks if the given type is `&str`.
pub(crate) fn is_str_ref_type(ty: &syn::Type) -> bool {
    let syn::Type::Reference(r) = ty else {
        return false;
    };

    is_str_type(&r.elem)
}

/// Checks if the given type is `String`.
pub(crate) fn is_string_type(ty: &syn::Type) -> bool {
    let syn::Type::Path(p) = ty else {
        return false;
    };

    p.path.is_ident("String")
}

/// Checks if the given type is `Cow<str>` or `Cow<'_, str>`.
pub(crate) fn is_cow_str_type(ty: &syn::Type) -> bool {
    extract_cow_type(ty).is_some_and(is_str_type)
}

/// Checks if the given type is string-like, i.e., one of `str`, `&str`, `String`, or `Cow<str>`.
pub(crate) fn is_string_like_type(ty: &syn::Type) -> bool {
    is_str_ref_type(ty) || is_str_type(ty) || is_string_type(ty) || is_cow_str_type(ty)
}

pub(crate) fn is_array_like_type(ty: &syn::Type) -> bool {
    extract_array_like_type(ty).is_some()
}

/// Extracts the element type from `[T; N]` or `[T]`.
pub(crate) fn extract_array_type(ty: &syn::Type) -> Option<&syn::Type> {
    match ty {
        syn::Type::Array(arr) => Some(&*arr.elem),
        syn::Type::Slice(slice) => Some(&*slice.elem),
        _ => None,
    }
}

/// Extracts the inner type from `Cow<T>`.
pub(crate) fn extract_cow_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(p) = ty else {
        return None;
    };
    let last = p.path.segments.last()?;
    if last.ident != "Cow" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };

    args.args.iter().find_map(|arg| match arg {
        syn::GenericArgument::Type(ty) => Some(ty),
        _ => None,
    })
}

/// Extracts the element type from `&[T]`.
pub(crate) fn extract_array_ref_type(ty: &syn::Type) -> Option<&syn::Type> {
    match ty {
        syn::Type::Reference(r) => extract_array_type(&r.elem),
        _ => None,
    }
}

/// Extracts the inner type from `Vec<T>`.
pub(crate) fn extract_vec_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(p) = ty else {
        return None;
    };
    let seg = p.path.segments.last()?;
    if seg.ident != "Vec" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    let Some(syn::GenericArgument::Type(ty)) = args.args.first() else {
        return None;
    };

    Some(ty)
}

/// Extracts the element type from `Cow<[T]>` or `Cow<[T; N]>`.
pub(crate) fn extract_cow_array_type(ty: &syn::Type) -> Option<&syn::Type> {
    extract_cow_type(ty).and_then(extract_array_type)
}

/// Extracts the element type from array-like types: `&[T]`, `[T; N]`, `[T]`, `Vec<T>`, or
/// `Cow<[T]>`.
pub(crate) fn extract_array_like_type(ty: &syn::Type) -> Option<&syn::Type> {
    extract_array_ref_type(ty)
        .or_else(|| extract_array_type(ty))
        .or_else(|| extract_vec_type(ty))
        .or_else(|| extract_cow_array_type(ty))
}

pub(crate) fn has_lifetime(ty: &syn::Type) -> bool {
    struct LifetimeChecker {
        has_lifetime: bool,
    }

    impl Visit<'_> for LifetimeChecker {
        fn visit_lifetime(&mut self, _i: &syn::Lifetime) {
            self.has_lifetime = true;
        }
    }

    let mut checker = LifetimeChecker {
        has_lifetime: false,
    };
    checker.visit_type(ty);
    checker.has_lifetime
}

/// Returns lifetime `'a`.
pub(crate) fn lifetime_a() -> syn::Lifetime {
    syn::Lifetime::new("'a", proc_macro2::Span::call_site())
}

/// Returns lifetime `'input`.
pub(crate) fn lifetime_input() -> syn::Lifetime {
    syn::Lifetime::new("'input", proc_macro2::Span::call_site())
}

/// Returns elided lifetime `'_`.
pub(crate) fn lifetime_elided() -> syn::Lifetime {
    syn::Lifetime::new("'_", proc_macro2::Span::call_site())
}

/// Rewrites every non-`'static` lifetime in `ty` to `lifetime`.
///
/// If `lifetime` is `None`, then every non-`'static` lifetime is rewritten to the elided lifetime
/// `'_`.
pub(crate) fn rename_lifetimes(mut ty: syn::Type, lifetime: Option<&syn::Lifetime>) -> syn::Type {
    struct Renamer<'a> {
        lifetime: Option<&'a syn::Lifetime>,
    }

    impl VisitMut for Renamer<'_> {
        fn visit_lifetime_mut(&mut self, i: &mut syn::Lifetime) {
            if i.ident != "static" {
                *i = match self.lifetime {
                    Some(lt) => lt.clone(),
                    None => lifetime_elided(),
                };
            }
        }

        fn visit_type_reference_mut(&mut self, i: &mut syn::TypeReference) {
            if let Some(ref mut lt) = i.lifetime {
                self.visit_lifetime_mut(lt);
            } else {
                i.lifetime = self.lifetime.cloned();
            }
            syn::visit_mut::visit_type_reference_mut(self, i);
        }
    }

    let mut v = Renamer { lifetime };
    v.visit_type_mut(&mut ty);
    ty
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_is_str_type() {
        let ty: syn::Type = parse_quote! { str };
        assert!(is_str_type(&ty));

        let ty: syn::Type = parse_quote! { String };
        assert!(!is_str_type(&ty));

        let ty: syn::Type = parse_quote! { &str };
        assert!(!is_str_type(&ty));
    }

    #[test]
    fn test_is_str_ref_type() {
        let ty: syn::Type = parse_quote! { &str };
        assert!(is_str_ref_type(&ty));

        let ty: syn::Type = parse_quote! { str };
        assert!(!is_str_ref_type(&ty));

        let ty: syn::Type = parse_quote! { &String };
        assert!(!is_str_ref_type(&ty));
    }

    #[test]
    fn test_is_string_type() {
        let ty: syn::Type = parse_quote! { String };
        assert!(is_string_type(&ty));

        let ty: syn::Type = parse_quote! { str };
        assert!(!is_string_type(&ty));

        let ty: syn::Type = parse_quote! { &String };
        assert!(!is_string_type(&ty));
    }

    #[test]
    fn test_is_cow_str_type() {
        let ty: syn::Type = parse_quote! { Cow<'_, str> };
        assert!(is_cow_str_type(&ty));

        let ty: syn::Type = parse_quote! { Cow<'a, str> };
        assert!(is_cow_str_type(&ty));

        let ty: syn::Type = parse_quote! { Cow<'_, String> };
        assert!(!is_cow_str_type(&ty));

        let ty: syn::Type = parse_quote! { Vec<str> };
        assert!(!is_cow_str_type(&ty));
    }

    #[test]
    fn test_is_string_like_type() {
        let ty: syn::Type = parse_quote! { str };
        assert!(is_string_like_type(&ty));

        let ty: syn::Type = parse_quote! { &str };
        assert!(is_string_like_type(&ty));

        let ty: syn::Type = parse_quote! { String };
        assert!(is_string_like_type(&ty));

        let ty: syn::Type = parse_quote! { Cow<'_, str> };
        assert!(is_string_like_type(&ty));

        let ty: syn::Type = parse_quote! { Vec<str> };
        assert!(!is_string_like_type(&ty));
    }

    #[test]
    fn test_extract_array_type() {
        let ty: syn::Type = parse_quote! { [i32; 5] };
        let elem = extract_array_type(&ty).unwrap();
        let expected: syn::Type = parse_quote! { i32 };
        assert_eq!(elem, &expected);

        let ty: syn::Type = parse_quote! { [i32] };
        let elem = extract_array_type(&ty).unwrap();
        let expected: syn::Type = parse_quote! { i32 };
        assert_eq!(elem, &expected);

        let ty: syn::Type = parse_quote! { Vec<i32> };
        assert!(extract_array_type(&ty).is_none());
    }

    #[test]
    fn test_extract_cow_type() {
        let ty: syn::Type = parse_quote! { Cow<'a, i32> };
        let inner = extract_cow_type(&ty).unwrap();
        let expected: syn::Type = parse_quote! { i32 };
        assert_eq!(inner, &expected);

        let ty: syn::Type = parse_quote! { Cow<i32> };
        let inner = extract_cow_type(&ty).unwrap();
        let expected: syn::Type = parse_quote! { i32 };
        assert_eq!(inner, &expected);

        let ty: syn::Type = parse_quote! { Vec<i32> };
        assert!(extract_cow_type(&ty).is_none());
    }

    #[test]
    fn test_extract_array_ref_type() {
        let ty: syn::Type = parse_quote! { &[i32] };
        let elem = extract_array_ref_type(&ty).unwrap();
        let expected: syn::Type = parse_quote! { i32 };
        assert_eq!(elem, &expected);

        let ty: syn::Type = parse_quote! { Vec<i32> };
        assert!(extract_array_ref_type(&ty).is_none());
    }

    #[test]
    fn test_extract_vec_type() {
        let ty: syn::Type = parse_quote! { Vec<i32> };
        let elem = extract_vec_type(&ty).unwrap();
        let expected: syn::Type = parse_quote! { i32 };
        assert_eq!(elem, &expected);

        let ty: syn::Type = parse_quote! { &[i32] };
        assert!(extract_vec_type(&ty).is_none());
    }

    #[test]
    fn test_extract_cow_array_type() {
        let ty: syn::Type = parse_quote! { Cow<'a, [i32]> };
        let elem = extract_cow_array_type(&ty).unwrap();
        let expected: syn::Type = parse_quote! { i32 };
        assert_eq!(elem, &expected);

        let ty: syn::Type = parse_quote! { Cow<'a, i32> };
        assert!(extract_cow_array_type(&ty).is_none());
    }

    #[test]
    fn test_extract_array_like_type() {
        let ty: syn::Type = parse_quote! { &[i32] };
        let elem = extract_array_like_type(&ty).unwrap();
        let expected: syn::Type = parse_quote! { i32 };
        assert_eq!(elem, &expected);

        let ty: syn::Type = parse_quote! { [i32; 5] };
        let elem = extract_array_like_type(&ty).unwrap();
        let expected: syn::Type = parse_quote! { i32 };
        assert_eq!(elem, &expected);

        let ty: syn::Type = parse_quote! { Vec<i32> };
        let elem = extract_array_like_type(&ty).unwrap();
        let expected: syn::Type = parse_quote! { i32 };
        assert_eq!(elem, &expected);

        let ty: syn::Type = parse_quote! { Cow<'a, [i32]> };
        let elem = extract_array_like_type(&ty).unwrap();
        let expected: syn::Type = parse_quote! { i32 };
        assert_eq!(elem, &expected);

        let ty: syn::Type = parse_quote! { i32 };
        assert!(extract_array_like_type(&ty).is_none());
    }

    #[test]
    fn test_has_lifetime() {
        let ty: syn::Type = parse_quote! { &str };
        assert!(!has_lifetime(&ty));

        let ty: syn::Type = parse_quote! { &'a str };
        assert!(has_lifetime(&ty));

        let ty: syn::Type = parse_quote! { Cow<'b, str> };
        assert!(has_lifetime(&ty));

        let ty: syn::Type = parse_quote! { (Vec<&'b str>, HashMap<&'static str, Cow<'c, str>>) };
        assert!(has_lifetime(&ty));
    }

    #[test]
    fn test_rename_lifetimes() {
        let lt: syn::Lifetime = parse_quote! { 'x };

        let ty: syn::Type = parse_quote! { &str };
        let renamed = rename_lifetimes(ty, Some(&lt));
        let expected: syn::Type = parse_quote! { &'x str };
        assert_eq!(renamed, expected);

        let ty = parse_quote! { &'static str };
        let renamed = rename_lifetimes(ty, Some(&lt));
        let expected: syn::Type = parse_quote! { &'static str };
        assert_eq!(renamed, expected);

        let ty = parse_quote! { Cow<'b, str> };
        let renamed = rename_lifetimes(ty, Some(&lt));
        let expected: syn::Type = parse_quote! { Cow<'x, str> };
        assert_eq!(renamed, expected);

        let ty = parse_quote! { (Vec<&'b str>, HashMap<&'static str, Cow<'c, str>>) };
        let renamed = rename_lifetimes(ty, Some(&lt));
        let expected: syn::Type =
            parse_quote! { (Vec<&'x str>, HashMap<&'static str, Cow<'x, str>>) };
        assert_eq!(renamed, expected);
    }
}
