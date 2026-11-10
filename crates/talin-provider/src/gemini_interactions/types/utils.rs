/// Generates `extra()` / `extra_mut()` accessors for a struct that has a private, flattened
/// `extra: BTreeMap<String, serde_json::Value>` field capturing undocumented API fields.
macro_rules! impl_extra_accessors {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl $ty {
                /// Returns the fields the server returned that this crate does not model.
                ///
                /// The map is empty for values you construct yourself.
                pub fn extra(&self) -> &::std::collections::BTreeMap<String, ::serde_json::Value> {
                    &self.extra
                }

                /// Returns a mutable reference to the fields the server returned that this
                /// crate does not model.
                pub fn extra_mut(
                    &mut self,
                ) -> &mut ::std::collections::BTreeMap<String, ::serde_json::Value> {
                    &mut self.extra
                }
            }
        )+
    };
}
pub(crate) use impl_extra_accessors;
