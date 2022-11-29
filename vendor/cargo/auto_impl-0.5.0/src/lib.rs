//! A proc-macro attribute for automatically implementing a trait for
//! references, some common smart pointers and closures.
//!
//! ## Simple example
//!
//! ```
//! use auto_impl::auto_impl;
//!
//! // This will generate two additional impl blocks: one `for &T` and one
//! // `for Box<T>` where `T: Foo`.
//! #[auto_impl(&, Box)]
//! trait Foo {
//!     fn foo(&self);
//! }
//!
//! impl Foo for i32 {
//!     fn foo(&self) {}
//! }
//!
//! fn requires_foo(_: impl Foo) {}
//!
//!
//! requires_foo(0i32);  // works: through the impl we defined above
//! requires_foo(&0i32); // works: through the generated impl
//! requires_foo(Box::new(0i32)); // works: through the generated impl
//! ```
//!
//!
//! # Basic syntax and supported types
//!
//! You can annotate your trait with the `#[auto_impl(...)]` attribute. That
//! attribute can only be used on traits and not on structs, enums or anything
//! else.
//!
//! In the attribute, you have to specify all so called *proxy types* (the
//! types you want to generate impls for) as a comma separated list. Each proxy
//! type has a short abbreviation that you have to list there.
//!
//! Currently the following proxy types are supported:
//!
//! | Abbreviation | Example generated impl |
//! | ------------ | ---------------------- |
//! | `&`          | `impl<T: Trait> Trait for &T` |
//! | `&mut`       | `impl<T: Trait> Trait for &mut T` |
//! | `Box`        | `impl<T: Trait> Trait for Box<T>` |
//! | `Rc`         | `impl<T: Trait> Trait for Rc<T>` |
//! | `Arc`        | `impl<T: Trait> Trait for Arc<T>` |
//! | `Fn`         | `impl<T: Fn()> Trait for T` |
//! | `FnMut`      | `impl<T: FnMut()> Trait for T` |
//! | `FnOnce`     | `impl<T: FnOnce()> Trait for T` |
//!
//!
//! # More examples
//!
//! More examples can be found in [the examples folder][examples]. In
//! particular, the `greet_closure` example shows how to use the `Fn*` proxy
//! types.
//!
//! [examples]: https://github.com/auto-impl-rs/auto_impl/tree/master/examples
//!
//! The following example shows that a trait can contain associated consts,
//! associated types and complex methods (with generics, bounds, ...).
//!
//! ```
//! use auto_impl::auto_impl;
//! use std::{fmt, rc::Rc};
//!
//!
//! #[auto_impl(&, &mut, Box, Rc)]
//! trait Animal {
//!     const NUMBER_OF_LEGS: u8;
//!
//!     type Name: fmt::Display;
//!     fn name(&self) -> Self::Name;
//!
//!     fn select_favorite<'a, I>(&self, toys: I) -> &'a str
//!     where
//!         I: Iterator<Item = &'a str>;
//! }
//!
//! struct Dog(String);
//!
//! impl Animal for Dog {
//!     const NUMBER_OF_LEGS: u8 = 4;
//!
//!     type Name = String;
//!     fn name(&self) -> Self::Name {
//!         self.0.clone()
//!     }
//!
//!     fn select_favorite<'a, I>(&self, mut toys: I) -> &'a str
//!     where
//!         I: Iterator<Item = &'a str>
//!     {
//!         toys.next().unwrap()
//!     }
//! }
//!
//! fn require_animal(_: impl Animal) {}
//!
//! // All these calls work, as the `#[auto_impl]` attribute generated four
//! // impls for all those proxy types
//! require_animal(Dog("Doggo".into()));
//! require_animal(&Dog("Doggo".into()));
//! require_animal(&mut Dog("Doggo".into()));
//! require_animal(Box::new(Dog("Doggo".into())));
//! require_animal(Rc::new(Dog("Doggo".into())));
//! ```
//!
//!
//! # Restriction of references and smart pointers
//!
//! Not every trait can be implemented for every proxy type. As an easy
//! example, consider this trait:
//!
//! ```
//! trait Bar {
//!     fn bar(&mut self);
//! }
//! ```
//!
//! If we try to implement it for immutable references via `#[auto_impl(&)]`
//! the following impl would be generated:
//!
//! ```ignore
//! impl<T: Bar> Bar for &T {
//!     fn bar(&mut self) {
//!         T::bar(*self)  // fails to compile
//!     }
//! }
//! ```
//!
//! As you can easily see, this won't work because we can't call `bar` through
//! an immutable reference. There are similar restrictions for many other
//! smart pointers and references.
//!
//! In the following table you can see which methods can be implemented for
//! which proxy type. If a trait contains at least one method that cannot be
//! implemented for a proxy type, you cannot implement the trait for that proxy
//! type.
//!
//! | Trait contains method with... | `&` | `&mut` | `Box` | `Rc` | `Arc` |
//! | ----------------------------- | --- | ------ | ----- | ---- | ----- |
//! | `&self` receiver              | ✔   | ✔      | ✔     | ✔    | ✔     |
//! | `&mut self` receiver          | ✗   | ✔      | ✔     | ✗    | ✗     |
//! | `self` receiver               | ✗   | ✗      | ✔     | ✗    | ✗     |
//! | no `self` receiver            | ✔   | ✔      | ✔     | ✔    | ✔     |
//!
//! References and smart pointers have **no restriction in regard to associated
//! types and associated consts**! Meaning: traits with associated types/consts
//! can always be implemented for references and smart pointers as long as the
//! methods of that trait can be implemented.
//!
//!
//! # Restriction of closure types (`Fn*` traits)
//!
//! The `Fn*` proxy types have a lot more restrictions than references and
//! smart pointer:
//! - the trait must not define any associated types or consts
//! - the trait must define **exactly one** method
//!     - the method must have a `self` receiver
//!     - the method must not return anything borrowed from `self`
//!     - the method must not have generic type or const parameters
//!
//! Additionally, some `Fn*` traits cannot be implemented for all `self`
//! receiver types:
//!
//! | `self` Receiver | `Fn` | `FnMut` | `FnOnce` |
//! | --------------- | ---- | ------- | -------- |
//! | `&self`         | ✔    | ✗       | ✗        |
//! | `&mut self`     | ✔    | ✔       | ✗        |
//! | `self`          | ✔    | ✔       | ✔        |
//!
//! Lastly, the impls generated for the `Fn*` proxy types contain `for T`. This
//! is the most general blanket impl. So just be aware of the problems with
//! coherence and orphan rules that can emerge due to this impl.
//!
//!
//! # The `keep_default_for` attribute for methods
//!
//! By default, the impls generated by `auto_impl` will overwrite all methods
//! of the trait, even those with default implementation. Sometimes, you want
//! to not overwrite default methods and instead use the default
//! implementation. You can do that by adding the
//! `#[auto_impl(keep_default_for(...))]` attribute to a default method. In the
//! parenthesis you need to list all proxy types for which the default method
//! should be kept.
//!
//! From [the `keep_default_for` example](
//! https://github.com/auto-impl-rs/auto_impl/blob/master/examples/keep_default_for.rs):
//!
//! ```
//! # use auto_impl::auto_impl;
//! #[auto_impl(&, Box)]
//! trait Foo {
//!     fn required(&self) -> String;
//!
//!     // The generated impl for `&T` will not override this method.
//!     #[auto_impl(keep_default_for(&))]
//!     fn provided(&self) {
//!         println!("Hello {}", self.required());
//!     }
//! }
//! ```
//!
//!
//! # The `nightly` feature gate
//!
//! By default, this crate compiles on stable. If you don't need stable support,
//! you can enable the `nightly` feature of this crate:
//!
//! ```toml
//! [dependencies]
//! auto_impl = { version = "*", features = ["nightly"] }
//! ```
//!
//! The nightly feature enables a few additional features that are not
//! available on stable yet. Currently, you get these advantages:
//! - All idents generated by auto_impl use the `def_site` hygiene and
//!   therefore will never ever have name collisions with user written idents.
//!   Note that `auto_impl` already (even without nightly feature) takes care
//!   that idents never collide, if possible. But `def_site` hygiene is still
//!   technically the more correct solution.

#![cfg_attr(
    feature = "nightly",
    feature(proc_macro_diagnostic, proc_macro_span, proc_macro_def_site)
)]


extern crate proc_macro;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{abort_call_site, proc_macro_error, set_dummy};
use quote::ToTokens;


mod analyze;
mod attr;
mod gen;
mod proxy;

/// See crate documentation for more information.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn auto_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    // Make sure that we emit a dummy in case of error
    let input: TokenStream2 = input.into();
    set_dummy(input.clone());

    // Try to parse the token stream from the attribute to get a list of proxy
    // types.
    let proxy_types = proxy::parse_types(args);

    match syn::parse2::<syn::ItemTrait>(input) {
        // The attribute was applied to a valid trait. Now it's time to execute
        // the main step: generate a token stream which contains an impl of the
        // trait for each proxy type.
        Ok(mut trait_def) => {
            let generated = gen::gen_impls(&proxy_types, &trait_def);

            // Before returning the trait definition, we have to remove all
            // `#[auto_impl(...)]` attributes on all methods.
            attr::remove_our_attrs(&mut trait_def);

            // We emit modified input instead of the original one
            // since it's better to remove our attributes even in case of error
            set_dummy(trait_def.to_token_stream());

            quote!(#trait_def #generated).into()
        }

        // If the token stream could not be parsed as trait, this most
        // likely means that the attribute was applied to a non-trait item.
        // Even if the trait definition was syntactically incorrect, the
        // compiler usually does some kind of error recovery to proceed. We
        // get the recovered tokens.
        Err(e) => abort_call_site!(
            "couldn't parse trait item";
            note = e;
            note = "the #[auto_impl] attribute can only be applied to traits!";
        ),
    }
}
