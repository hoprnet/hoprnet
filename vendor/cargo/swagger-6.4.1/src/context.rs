//! Module for API context management.
//!
//! This module defines traits and structs that can be used  to manage
//! contextual data related to a request, as it is passed through a series of
//! hyper services.
//!
//! See the `context_tests` module below for examples of how to use.

use crate::auth::{AuthData, Authorization};
use crate::XSpanIdString;
use hyper::{service::Service, Request, Response};
use std::future::Future;
use std::pin::Pin;

/// Defines methods for accessing, modifying, adding and removing the data stored
/// in a context. Used to specify the requirements that a hyper service makes on
/// a generic context type that it receives with a request, e.g.
///
/// ```rust
/// # use futures::future::ok;
/// # use std::future::Future;
/// # use std::marker::PhantomData;
/// # use std::pin::Pin;
/// # use std::task::{Context, Poll};
/// # use swagger::context::*;
/// #
/// # struct MyItem;
/// # fn do_something_with_my_item(item: &MyItem) {}
/// #
/// struct MyService<C> {
///     marker: PhantomData<C>,
/// }
///
/// impl<C> hyper::service::Service<(hyper::Request<hyper::Body>, C)> for MyService<C>
///     where C: Has<MyItem> + Send + 'static
/// {
///     type Response = hyper::Response<hyper::Body>;
///     type Error = std::io::Error;
///     type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>>>>;
///
///     fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
///         Poll::Ready(Ok(()))
///     }
///
///     fn call(&mut self, req : (hyper::Request<hyper::Body>, C)) -> Self::Future {
///         let (_, context) = req;
///         do_something_with_my_item(Has::<MyItem>::get(&context));
///         Box::pin(ok(hyper::Response::new(hyper::Body::empty())))
///     }
/// }
/// ```
pub trait Has<T> {
    /// Get an immutable reference to the value.
    fn get(&self) -> &T;
    /// Get a mutable reference to the value.
    fn get_mut(&mut self) -> &mut T;
    /// Set the value.
    fn set(&mut self, value: T);
}

/// Defines a method for permanently extracting a value, changing the resulting
/// type. Used to specify that a hyper service consumes some data from the context,
/// making it unavailable to later layers, e.g.
///
/// ```rust
/// # use futures::future::{Future, ok};
/// # use std::task::{Context, Poll};
/// # use std::marker::PhantomData;
/// # use swagger::context::*;
/// #
/// struct MyItem1;
/// struct MyItem2;
/// struct MyItem3;
///
/// struct MiddlewareService<T, C> {
///     inner: T,
///     marker: PhantomData<C>,
/// }
///
/// impl<T, C, D, E> hyper::service::Service<(hyper::Request<hyper::Body>, C)> for MiddlewareService<T, C>
///     where
///         C: Pop<MyItem1, Result=D> + Send + 'static,
///         D: Pop<MyItem2, Result=E>,
///         E: Pop<MyItem3>,
///         E::Result: Send + 'static,
///         T: hyper::service::Service<(hyper::Request<hyper::Body>, E::Result)>
/// {
///     type Response = T::Response;
///     type Error = T::Error;
///     type Future = T::Future;
///
///     fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
///         self.inner.poll_ready(cx)
///     }
///
///     fn call(&mut self, req : (hyper::Request<hyper::Body>, C)) -> Self::Future {
///         let (request, context) = req;
///
///         // type annotations optional, included for illustrative purposes
///         let (_, context): (MyItem1, D) = context.pop();
///         let (_, context): (MyItem2, E) = context.pop();
///         let (_, context): (MyItem3, E::Result) = context.pop();
///
///         self.inner.call((request, context))
///     }
/// }
pub trait Pop<T> {
    /// The type that remains after the value has been popped.
    type Result;
    /// Extracts a value.
    fn pop(self) -> (T, Self::Result);
}

/// Defines a method for inserting a value, changing the resulting
/// type. Used to specify that a hyper service adds some data from the context,
/// making it available to later layers, e.g.
///
/// ```rust
/// # use swagger::context::*;
/// # use std::marker::PhantomData;
/// # use std::task::{Context, Poll};
/// #
/// struct MyItem1;
/// struct MyItem2;
/// struct MyItem3;
///
/// struct MiddlewareService<T, C> {
///     inner: T,
///     marker: PhantomData<C>,
/// }
///
/// impl<T, C, D, E> hyper::service::Service<(hyper::Request<hyper::Body>, C)> for MiddlewareService<T, C>
///     where
///         C: Push<MyItem1, Result=D> + Send + 'static,
///         D: Push<MyItem2, Result=E>,
///         E: Push<MyItem3>,
///         E::Result: Send + 'static,
///         T: hyper::service::Service<(hyper::Request<hyper::Body>, E::Result)>
/// {
///     type Response = T::Response;
///     type Error = T::Error;
///     type Future = T::Future;
///
///     fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
///         self.inner.poll_ready(cx)
///     }
///
///     fn call(&mut self, req : (hyper::Request<hyper::Body>, C)) -> Self::Future {
///         let (request, context) = req;
///         let context = context
///             .push(MyItem1{})
///             .push(MyItem2{})
///             .push(MyItem3{});
///         self.inner.call((request, context))
///     }
/// }
pub trait Push<T> {
    /// The type that results from adding an item.
    type Result;
    /// Inserts a value.
    fn push(self, value: T) -> Self::Result;
}

/// Defines a struct that can be used to build up contexts recursively by
/// adding one item to the context at a time, and a unit struct representing an
/// empty context. The first argument is the name of the newly defined context struct
/// that is used to add an item to the context, the second argument is the name of
/// the empty context struct, and subsequent arguments are the types
/// that can be stored in contexts built using these struct.
///
/// A cons list built using the generated context type will implement `Has<T>` and `Pop<T>`
/// for each type T that appears in the list, provided that the list only
/// contains the types that were passed to the macro invocation after the context
/// type name.
///
/// All list types constructed using the generated types will implement `Push<T>`
/// for all types `T` that appear in the list passed to the macro invocation.
///
/// E.g.
///
/// ```edition2018
/// #[derive(Default)]
/// struct MyType1;
/// #[derive(Default)]
/// struct MyType2;
/// #[derive(Default)]
/// struct MyType3;
/// #[derive(Default)]
/// struct MyType4;
///
/// swagger::new_context_type!(MyContext, MyEmpContext, MyType1, MyType2, MyType3);
///
/// fn use_has_my_type_1<T: swagger::Has<MyType1>> (_: &T) {}
/// fn use_has_my_type_2<T: swagger::Has<MyType2>> (_: &T) {}
/// fn use_has_my_type_3<T: swagger::Has<MyType3>> (_: &T) {}
/// fn use_has_my_type_4<T: swagger::Has<MyType4>> (_: &T) {}
///
/// // Will implement `Has<MyType1>` and `Has<MyType2>` because these appear
/// // in the type, and were passed to `new_context_type!`. Will not implement
/// // `Has<MyType3>` even though it was passed to `new_context_type!`, because
/// // it is not included in the type.
/// type ExampleContext = MyContext<MyType1, MyContext<MyType2,  MyEmpContext>>;
///
/// // Will not implement `Has<MyType4>` even though it appears in the type,
/// // because `MyType4` was not passed to `new_context_type!`.
/// type BadContext = MyContext<MyType1, MyContext<MyType4, MyEmpContext>>;
///
/// fn main() {
///     # use swagger::Push as _;
///     let context : ExampleContext =
///         MyEmpContext::default()
///             .push(MyType2{})
///             .push(MyType1{});
///
///     use_has_my_type_1(&context);
///     use_has_my_type_2(&context);
///     // use_has_my_type_3(&context);      // will fail
///
///     // Will fail because `MyType4`// was not passed to `new_context_type!`
///     // let context = MyEmpContext::default().push(MyType4{});
///
///     let bad_context: BadContext = BadContext::default();
///     // use_has_my_type_4(&bad_context);  // will fail
/// }
/// ```
///
/// See the `context_tests` module for more usage examples.
#[macro_export]
macro_rules! new_context_type {
    ($context_name:ident, $empty_context_name:ident, $($types:ty),+ ) => {

        /// Wrapper type for building up contexts recursively, adding one item
        /// to the context at a time.
        #[derive(Debug, Clone, Default, PartialEq, Eq)]
        pub struct $context_name<T, C> {
            head: T,
            tail: C,
        }

        /// Unit struct representing an empty context with no data in it.
        #[derive(Debug, Clone, Default, PartialEq, Eq)]
        pub struct $empty_context_name;

        // implement `Push<T>` on the empty context type for each type `T` that
        // was passed to the macro
        $(
        impl $crate::Push<$types> for $empty_context_name {
            type Result = $context_name<$types, Self>;
            fn push(self, item: $types) -> Self::Result {
                $context_name{head: item, tail: Self::default()}
            }
        }

        // implement `Has<T>` for a list where `T` is the type of the head
        impl<C> $crate::Has<$types> for $context_name<$types, C> {
            fn set(&mut self, item: $types) {
                self.head = item;
            }

            fn get(&self) -> &$types {
                &self.head
            }

            fn get_mut(&mut self) -> &mut $types {
                &mut self.head
            }
        }

        // implement `Pop<T>` for a list where `T` is the type of the head
        impl<C> $crate::Pop<$types> for $context_name<$types, C> {
            type Result = C;
            fn pop(self) -> ($types, Self::Result) {
                (self.head, self.tail)
            }
        }

        // implement `Push<U>` for non-empty lists, for each type `U` that was passed
        // to the macro
        impl<C, T> $crate::Push<$types> for $context_name<T, C> {
            type Result = $context_name<$types, Self>;
            fn push(self, item: $types) -> Self::Result {
                $context_name{head: item, tail: self}
            }
        }
        )+

        // Add implementations of `Has<T>` and `Pop<T>` when `T` is any type stored in
        // the list, not just the head.
        $crate::new_context_type!(impl extend_has $context_name, $empty_context_name, $($types),+);
    };

    // "HELPER" MACRO CASE - NOT FOR EXTERNAL USE
    // takes a type `Type1` ($head) and a non-empty list of types `Types` ($tail). First calls
    // another helper macro to define the following impls, for each `Type2` in `Types`:
    // ```
    // impl<C: Has<Type1> Has<Type1> for $context_name<Type2, C> {...}
    // impl<C: Has<Type2> Has<Type2> for $context_name<Type1, C> {...}
    // impl<C: Pop<Type1> Pop<Type1> for $context_name<Type2, C> {...}
    // impl<C: Pop<Type2> Pop<Type2> for $context_name<Type1, C> {...}
    // ```
    // then calls itself again with the rest of the list. The end result is to define the above
    // impls for all distinct pairs of types in the original list.
    (impl extend_has $context_name:ident, $empty_context_name:ident, $head:ty, $($tail:ty),+ ) => {

        $crate::new_context_type!(
            impl extend_has_helper
            $context_name,
            $empty_context_name,
            $head,
            $($tail),+
        );
        $crate::new_context_type!(impl extend_has $context_name, $empty_context_name, $($tail),+);
    };

    // "HELPER" MACRO CASE - NOT FOR EXTERNAL USE
    // base case of the preceding helper macro - was passed an empty list of types, so
    // we don't need to do anything.
    (impl extend_has $context_name:ident, $empty_context_name:ident, $head:ty) => {};

    // "HELPER" MACRO CASE - NOT FOR EXTERNAL USE
    // takes a type `Type1` ($type) and a non-empty list of types `Types` ($types). For
    // each `Type2` in `Types`, defines the following impls:
    // ```
    // impl<C: Has<Type1> Has<Type1> for $context_name<Type2, C> {...}
    // impl<C: Has<Type2> Has<Type2> for $context_name<Type1, C> {...}
    // impl<C: Pop<Type1> Pop<Type1> for $context_name<Type2, C> {...}
    // impl<C: Pop<Type2> Pop<Type2> for $context_name<Type1, C> {...}
    // ```
    //
    (impl extend_has_helper
        $context_name:ident,
        $empty_context_name:ident,
        $type:ty,
        $($types:ty),+
        ) => {
        $(
            impl<C: $crate::Has<$type>> $crate::Has<$type> for $context_name<$types, C> {
                fn set(&mut self, item: $type) {
                    self.tail.set(item);
                }

                fn get(&self) -> &$type {
                    self.tail.get()
                }

                fn get_mut(&mut self) -> &mut $type {
                    self.tail.get_mut()
                }
            }

            impl<C: $crate::Has<$types>> $crate::Has<$types> for $context_name<$type, C> {
                fn set(&mut self, item: $types) {
                    self.tail.set(item);
                }

                fn get(&self) -> &$types {
                    self.tail.get()
                }

                fn get_mut(&mut self) -> &mut $types {
                    self.tail.get_mut()
                }
            }

            impl<C> $crate::Pop<$type> for $context_name<$types, C> where C: $crate::Pop<$type> {
                type Result = $context_name<$types, C::Result>;
                fn pop(self) -> ($type, Self::Result) {
                    let (value, tail) = self.tail.pop();
                    (value, $context_name{ head: self.head, tail})
                }
            }

            impl<C> $crate::Pop<$types> for $context_name<$type, C> where C: $crate::Pop<$types> {
                type Result = $context_name<$type, C::Result>;
                fn pop(self) -> ($types, Self::Result) {
                    let (value, tail) = self.tail.pop();
                    (value, $context_name{ head: self.head, tail})
                }
            }
        )+
    };
}

// Create a default context type to export.
new_context_type!(
    ContextBuilder,
    EmptyContext,
    XSpanIdString,
    Option<AuthData>,
    Option<Authorization>
);

/// Macro for easily defining context types. The first argument should be a
/// context type created with `new_context_type!` and subsequent arguments are the
/// types to be stored in the context, with the outermost first.
///
/// ```rust
/// # #[macro_use] extern crate swagger;
/// # use swagger::{Has, Pop, Push};
///
/// # struct Type1;
/// # struct Type2;
/// # struct Type3;
///
/// # new_context_type!(MyContext, MyEmptyContext, Type1, Type2, Type3);
///
/// // the following two types are identical
/// type ExampleContext1 = make_context_ty!(MyContext, MyEmptyContext, Type1, Type2, Type3);
/// type ExampleContext2 = MyContext<Type1, MyContext<Type2, MyContext<Type3, MyEmptyContext>>>;
///
/// // e.g. this wouldn't compile if they were different types
/// fn do_nothing(input: ExampleContext1) -> ExampleContext2 {
///     input
/// }
/// ```
#[macro_export]
macro_rules! make_context_ty {
    ($context_name:ident, $empty_context_name:ident, $type:ty $(, $types:ty)* $(,)* ) => {
        $context_name<$type, $crate::make_context_ty!($context_name, $empty_context_name, $($types),*)>
    };
    ($context_name:ident, $empty_context_name:ident $(,)* ) => {
        $empty_context_name
    };
}

/// Macro for easily defining context values. The first argument should be a
/// context type created with `new_context_type!` and subsequent arguments are the
/// values to be stored in the context, with the outermost first.
///
/// ```rust
/// # #[macro_use] extern crate swagger;
/// # use swagger::{Has, Pop, Push};
///
/// # #[derive(PartialEq, Eq, Debug)]
/// # struct Type1;
/// # #[derive(PartialEq, Eq, Debug)]
/// # struct Type2;
/// # #[derive(PartialEq, Eq, Debug)]
/// # struct Type3;
///
/// # new_context_type!(MyContext, MyEmptyContext, Type1, Type2, Type3);
///
/// fn main() {
///     // the following are equivalent
///     let context1 = make_context!(MyContext, MyEmptyContext, Type1 {}, Type2 {}, Type3 {});
///     let context2 = MyEmptyContext::default()
///         .push(Type3{})
///         .push(Type2{})
///         .push(Type1{});
///
///     assert_eq!(context1, context2);
/// }
/// ```
#[macro_export]
macro_rules! make_context {
    ($context_name:ident, $empty_context_name:ident, $value:expr $(, $values:expr)* $(,)*) => {
        $crate::make_context!($context_name, $empty_context_name, $($values),*).push($value)
    };
    ($context_name:ident, $empty_context_name:ident $(,)* ) => {
        $empty_context_name::default()
    };
}

/// Context wrapper, to bind an API with a context.
#[derive(Debug)]
pub struct ContextWrapper<T, C> {
    api: T,
    context: C,
}

impl<T, C> ContextWrapper<T, C> {
    /// Create a new ContextWrapper, binding the API and context.
    pub fn new(api: T, context: C) -> Self {
        Self { api, context }
    }

    /// Borrows the API.
    pub fn api(&self) -> &T {
        &self.api
    }

    /// Borrows the context.
    pub fn context(&self) -> &C {
        &self.context
    }
}

impl<T: Clone, C: Clone> Clone for ContextWrapper<T, C> {
    fn clone(&self) -> Self {
        ContextWrapper {
            api: self.api.clone(),
            context: self.context.clone(),
        }
    }
}

/// Trait designed to ensure consistency in context used by swagger middlewares
///
/// ```rust
/// # use swagger::context::*;
/// # use std::marker::PhantomData;
/// # use std::task::{Context, Poll};
/// # use swagger::auth::{AuthData, Authorization};
/// # use swagger::XSpanIdString;
///
/// struct ExampleMiddleware<T, C> {
///     inner: T,
///     marker: PhantomData<C>,
/// }
///
/// impl<T, C> hyper::service::Service<(hyper::Request<hyper::Body>, C)> for ExampleMiddleware<T, C>
///     where
///         T: SwaggerService<hyper::Body, hyper::Body, C>,
///         C: Has<Option<AuthData>> +
///            Has<Option<Authorization>> +
///            Has<XSpanIdString> +
///            Clone +
///            Send +
///            'static,
/// {
///     type Response = T::Response;
///     type Error = T::Error;
///     type Future = T::Future;
///
///     fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
///         self.inner.poll_ready(cx)
///     }
///
///     fn call(&mut self, req: (hyper::Request<hyper::Body>, C)) -> Self::Future {
///         self.inner.call(req)
///     }
/// }
/// ```
pub trait SwaggerService<RequestBody, ResponseBody, Context>:
    Clone
    + Service<
        (Request<RequestBody>, Context),
        Response = Response<ResponseBody>,
        Error = hyper::Error,
        Future = Pin<Box<dyn Future<Output = Result<Response<ResponseBody>, hyper::Error>>>>,
    >
where
    Context: Has<Option<AuthData>>
        + Has<Option<Authorization>>
        + Has<XSpanIdString>
        + Clone
        + 'static
        + Send,
{
}

impl<ReqB, ResB, Context, T> SwaggerService<ReqB, ResB, Context> for T
where
    T: Clone
        + Service<
            (Request<ReqB>, Context),
            Response = Response<ResB>,
            Error = hyper::Error,
            Future = Pin<Box<dyn Future<Output = Result<Response<ResB>, hyper::Error>>>>,
        >,
    Context: Has<Option<AuthData>>
        + Has<Option<Authorization>>
        + Has<XSpanIdString>
        + Clone
        + 'static
        + Send,
{
}

#[cfg(test)]
mod context_tests {
    use super::Has;
    use super::*;

    struct ContextItem1 {
        val: u32,
    }
    struct ContextItem2;
    struct ContextItem3;

    // all the hyper service layers you will be using, and what requirements
    // their contexts types have. Use the `new_context_type!` macro to create
    // a context type and empty context type that are capable of containing all the
    // types that your hyper services require.
    new_context_type!(
        MyContext,
        MyEmptyContext,
        ContextItem1,
        ContextItem2,
        ContextItem3
    );

    #[test]
    fn send_request() {
        let t = MyEmptyContext::default();

        let t = t.push(ContextItem1 { val: 1 });
        let t = t.push(ContextItem2);

        {
            let v: &ContextItem1 = t.get();
            assert_eq!(v.val, 1);
        }

        let (_, mut t): (ContextItem2, _) = t.pop();

        {
            let v: &mut ContextItem1 = t.get_mut();
            v.val = 4;
        }

        {
            let v: &ContextItem1 = t.get();
            assert_eq!(v.val, 4);
        }
    }
}
