pub(crate) fn filter_attrs<'a>(
    attrs: &'a [syn::Attribute],
    names: &'a [&'a str],
) -> impl Iterator<Item = &'a syn::Attribute> + 'a {
    attrs.iter().filter(move |attr| {
        for filter in names {
            if attr.path().is_ident(filter) {
                return true;
            }
        }
        false
    })
}
