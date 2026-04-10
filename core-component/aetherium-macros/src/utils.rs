use syn::ItemFn;

pub fn is_async(func: &ItemFn) -> bool {
    func.sig.asyncness.is_some()
}
