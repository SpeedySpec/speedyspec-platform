//! `#[port(key = "PHP\\Interface\\Name")]` (or `#[port(key = "…", edge)]`) on a port trait in a contracts crate
//! generates, with nothing from PHP:
//!
//! - the trait, unchanged;
//! - `<TRAIT>_KEY`, the key the port is bound under (its PHP twin's interface name);
//! - `Proxy<Trait>`, which implements the trait by forwarding every call to whatever the container binds under the
//!   key now, through a [`PortCell`](../speedyspec_container/struct.PortCell.html). Providers hand proxies to other
//!   seams, so a rebind (from Rust configuration or, for an edge port, from PHP) reaches every caller;
//! - for an **edge** port (one whose argument and result types convert to PHP), a hidden exported
//!   `__speedyspec_port_<Trait>!` macro that carries the trait's key, path and method signatures to a callback
//!   macro, so the seam's `-php` crate can generate the twin and PHP adapter with
//!   `contracts::__speedyspec_port_<Trait>!(::speedyspec_php_port_macros::php_twin);`. The module path inside the
//!   contracts crate is the crate root unless `path = "module::sub"` is given.
//!
//! Every method must take `&self`, and the trait must stay object-safe (no generic methods, no `impl Trait`, no
//! `async fn`, no associated items). The generated code names `::speedyspec_container`, so the contracts crate
//! depends on it.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    FnArg, Ident, ItemTrait, LitStr, Pat, ReturnType, TraitItem, Type, parse_macro_input,
    visit::Visit,
};

#[proc_macro_attribute]
pub fn port(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut key: Option<LitStr> = None;
    let mut path: Option<LitStr> = None;
    let mut edge = false;
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("key") {
            key = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("path") {
            path = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("edge") {
            edge = true;
            Ok(())
        } else {
            Err(meta.error("expected `key = \"…\"`, `edge` or `path = \"module::sub\"`"))
        }
    });
    parse_macro_input!(attr with parser);
    let item = parse_macro_input!(item as ItemTrait);
    let Some(key) = key else {
        return syn::Error::new_spanned(
            &item.ident,
            "#[port] needs key = \"PHP\\\\Interface\\\\Name\"",
        )
        .to_compile_error()
        .into();
    };
    match expand(&key, path.as_ref(), edge, &item) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand(
    key: &LitStr,
    path: Option<&LitStr>,
    edge: bool,
    item: &ItemTrait,
) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &item.ident;
    let vis = &item.vis;
    if !item.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &item.generics,
            "#[port] traits cannot be generic",
        ));
    }
    let proxy = format_ident!("Proxy{}", ident);
    let key_const = Ident::new(
        &format!("{}_KEY", screaming(&ident.to_string())),
        ident.span(),
    );
    let mut forwards = Vec::new();
    let mut signatures = Vec::new();

    for entry in &item.items {
        let TraitItem::Fn(method) = entry else {
            return Err(syn::Error::new_spanned(
                entry,
                "#[port] traits hold only methods",
            ));
        };
        let sig = &method.sig;
        let name = &sig.ident;
        if !sig.generics.params.is_empty() {
            return Err(syn::Error::new_spanned(
                sig,
                format!("#[port] methods cannot be generic: {name}"),
            ));
        }
        if sig.asyncness.is_some() || has_impl_trait(sig) {
            return Err(syn::Error::new_spanned(
                sig,
                format!("#[port] methods must be object-safe: {name}"),
            ));
        }
        let mut receiver = false;
        let mut args = Vec::new();
        for input in &sig.inputs {
            match input {
                FnArg::Receiver(r) if r.reference.is_some() && r.mutability.is_none() => {
                    receiver = true
                }
                FnArg::Receiver(r) => {
                    return Err(syn::Error::new_spanned(r, "#[port] methods take &self"));
                }
                FnArg::Typed(typed) => {
                    let Pat::Ident(pat) = typed.pat.as_ref() else {
                        return Err(syn::Error::new_spanned(
                            &typed.pat,
                            "#[port] arguments need plain names",
                        ));
                    };
                    args.push(pat.ident.clone());
                }
            }
        }
        if !receiver {
            return Err(syn::Error::new_spanned(sig, "#[port] methods take &self"));
        }
        let output = match &sig.output {
            ReturnType::Default => quote!(),
            ReturnType::Type(arrow, ty) => quote!(#arrow #ty),
        };
        let inputs = &sig.inputs;
        let unsafety = &sig.unsafety;
        forwards.push(quote! {
            #unsafety fn #name(#inputs) #output {
                self.0.with(|port| port.#name(#(#args),*))
            }
        });
        signatures.push(quote!(#sig;));
    }

    let signature_macro = if edge {
        let macro_name = format_ident!("__speedyspec_port_{}", ident);
        let module = match path {
            Some(path) => {
                let module: syn::Path = path.parse()?;
                quote!(#module ::)
            }
            None => quote!(),
        };
        let dollar = proc_macro2::Punct::new('$', proc_macro2::Spacing::Alone);
        let callback = Ident::new("callback", Span::call_site());
        let krate = Ident::new("crate", Span::call_site());
        quote! {
            #[doc(hidden)]
            #[macro_export]
            macro_rules! #macro_name {
                (#dollar (#dollar #callback:tt)*) => {
                    #dollar (#dollar #callback)* ! {
                        key = #key,
                        path = #dollar #krate :: #module #ident,
                        trait #ident { #(#signatures)* }
                    }
                };
            }
        }
    } else {
        quote!()
    };

    Ok(quote! {
        #item

        /// The key this port is bound under in the container: its PHP twin's interface name.
        #vis const #key_const: &str = #key;

        /// Forwards every call to whatever the container binds under this port's key now.
        #vis struct #proxy(::speedyspec_container::PortCell<dyn #ident>);

        impl #proxy {
            pub fn new(container: &::std::rc::Rc<::speedyspec_container::Container>) -> Self {
                Self(::speedyspec_container::PortCell::new(container, #key_const))
            }

            /// The cell this proxy forwards through.
            pub fn cell(&self) -> &::speedyspec_container::PortCell<dyn #ident> {
                &self.0
            }
        }

        impl #ident for #proxy {
            #(#forwards)*
        }

        #signature_macro
    })
}

/// `HookApi` → `HOOK_API`.
fn screaming(name: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = name.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if c.is_uppercase() && i > 0 {
            let prev_lower = chars[i - 1].is_lowercase() || chars[i - 1].is_ascii_digit();
            let next_lower = chars.get(i + 1).is_some_and(|n| n.is_lowercase());
            if prev_lower || (chars[i - 1].is_uppercase() && next_lower) {
                out.push('_');
            }
        }
        out.extend(c.to_uppercase());
    }
    out
}

fn has_impl_trait(sig: &syn::Signature) -> bool {
    struct Finder(bool);
    impl<'ast> Visit<'ast> for Finder {
        fn visit_type(&mut self, ty: &'ast Type) {
            if matches!(ty, Type::ImplTrait(_)) {
                self.0 = true;
            }
            syn::visit::visit_type(self, ty);
        }
    }
    let mut finder = Finder(false);
    finder.visit_signature(sig);
    finder.0
}
