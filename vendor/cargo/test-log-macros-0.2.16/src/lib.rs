// Copyright (C) 2019-2023 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as Tokens;

use quote::quote;

use syn::parse::Parse;
use syn::parse_macro_input;
use syn::Attribute;
use syn::Expr;
use syn::ItemFn;
use syn::Lit;
use syn::Meta;


#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
  let item = parse_macro_input!(item as ItemFn);
  try_test(attr, item)
    .unwrap_or_else(syn::Error::into_compile_error)
    .into()
}

fn parse_attrs(attrs: Vec<Attribute>) -> syn::Result<(AttributeArgs, Vec<Attribute>)> {
  let mut attribute_args = AttributeArgs::default();
  if cfg!(feature = "unstable") {
    let mut ignored_attrs = vec![];
    for attr in attrs {
      let matched = attribute_args.try_parse_attr_single(&attr)?;
      // Keep only attrs that didn't match the #[test_log(_)] syntax.
      if !matched {
        ignored_attrs.push(attr);
      }
    }

    Ok((attribute_args, ignored_attrs))
  } else {
    Ok((attribute_args, attrs))
  }
}

fn try_test(attr: TokenStream, input: ItemFn) -> syn::Result<Tokens> {
  let inner_test = if attr.is_empty() {
    quote! { ::core::prelude::v1::test }
  } else {
    attr.into()
  };

  let ItemFn {
    attrs,
    vis,
    sig,
    block,
  } = input;

  let (attribute_args, ignored_attrs) = parse_attrs(attrs)?;
  let logging_init = expand_logging_init(&attribute_args);
  let tracing_init = expand_tracing_init(&attribute_args);

  let result = quote! {
    #[#inner_test]
    #(#ignored_attrs)*
    #vis #sig {
      // We put all initialization code into a separate module here in
      // order to prevent potential ambiguities that could result in
      // compilation errors. E.g., client code could use traits that
      // could have methods that interfere with ones we use as part of
      // initialization; with a `Foo` trait that is implemented for T
      // and that contains a `map` (or similarly common named) method
      // that could cause an ambiguity with `Iterator::map`, for
      // example.
      // The alternative would be to use fully qualified call syntax in
      // all initialization code, but that's much harder to control.
      mod init {
        pub fn init() {
          #logging_init
          #tracing_init
        }
      }

      init::init();

      #block
    }
  };
  Ok(result)
}


#[derive(Debug, Default)]
struct AttributeArgs {
  default_log_filter: Option<String>,
}

impl AttributeArgs {
  fn try_parse_attr_single(&mut self, attr: &Attribute) -> syn::Result<bool> {
    if !attr.path().is_ident("test_log") {
      return Ok(false)
    }

    let nested_meta = attr.parse_args_with(Meta::parse)?;
    let name_value = if let Meta::NameValue(name_value) = nested_meta {
      name_value
    } else {
      return Err(syn::Error::new_spanned(
        &nested_meta,
        "Expected NameValue syntax, e.g. 'default_log_filter = \"debug\"'.",
      ))
    };

    let ident = if let Some(ident) = name_value.path.get_ident() {
      ident
    } else {
      return Err(syn::Error::new_spanned(
        &name_value.path,
        "Expected NameValue syntax, e.g. 'default_log_filter = \"debug\"'.",
      ))
    };

    let arg_ref = if ident == "default_log_filter" {
      &mut self.default_log_filter
    } else {
      return Err(syn::Error::new_spanned(
        &name_value.path,
        "Unrecognized attribute, see documentation for details.",
      ))
    };

    if let Expr::Lit(lit) = &name_value.value {
      if let Lit::Str(lit_str) = &lit.lit {
        *arg_ref = Some(lit_str.value());
      }
    }

    // If we couldn't parse the value on the right-hand side because it was some
    // unexpected type, e.g. #[test_log::log(default_log_filter=10)], return an error.
    if arg_ref.is_none() {
      return Err(syn::Error::new_spanned(
        &name_value.value,
        "Failed to parse value, expected a string",
      ))
    }

    Ok(true)
  }
}


/// Expand the initialization code for the `log` crate.
#[cfg(all(feature = "log", not(feature = "trace")))]
fn expand_logging_init(attribute_args: &AttributeArgs) -> Tokens {
  let add_default_log_filter = if let Some(default_log_filter) = &attribute_args.default_log_filter
  {
    quote! {
      let env_logger_builder = env_logger_builder
        .parse_env(::test_log::env_logger::Env::default().default_filter_or(#default_log_filter));
    }
  } else {
    quote! {}
  };

  quote! {
    {
      let mut env_logger_builder = ::test_log::env_logger::builder();
      #add_default_log_filter
      let _ = env_logger_builder.is_test(true).try_init();
    }
  }
}

#[cfg(not(all(feature = "log", not(feature = "trace"))))]
fn expand_logging_init(_attribute_args: &AttributeArgs) -> Tokens {
  quote! {}
}

/// Expand the initialization code for the `tracing` crate.
#[cfg(feature = "trace")]
fn expand_tracing_init(attribute_args: &AttributeArgs) -> Tokens {
  let env_filter = if let Some(default_log_filter) = &attribute_args.default_log_filter {
    quote! {
      ::test_log::tracing_subscriber::EnvFilter::builder()
        .with_default_directive(
          #default_log_filter
            .parse()
            .expect("test-log: default_log_filter must be valid")
        )
        .from_env_lossy()
    }
  } else {
    quote! { ::test_log::tracing_subscriber::EnvFilter::from_default_env() }
  };

  quote! {
    {
      let __internal_event_filter = {
        use ::test_log::tracing_subscriber::fmt::format::FmtSpan;

        match ::std::env::var_os("RUST_LOG_SPAN_EVENTS") {
          Some(mut value) => {
            value.make_ascii_lowercase();
            let value = value.to_str().expect("test-log: RUST_LOG_SPAN_EVENTS must be valid UTF-8");
            value
              .split(",")
              .map(|filter| match filter.trim() {
                "new" => FmtSpan::NEW,
                "enter" => FmtSpan::ENTER,
                "exit" => FmtSpan::EXIT,
                "close" => FmtSpan::CLOSE,
                "active" => FmtSpan::ACTIVE,
                "full" => FmtSpan::FULL,
                _ => panic!("test-log: RUST_LOG_SPAN_EVENTS must contain filters separated by `,`.\n\t\
                  For example: `active` or `new,close`\n\t\
                  Supported filters: new, enter, exit, close, active, full\n\t\
                  Got: {}", value),
              })
              .fold(FmtSpan::NONE, |acc, filter| filter | acc)
          },
          None => FmtSpan::NONE,
        }
      };

      let _ = ::test_log::tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(#env_filter)
        .with_span_events(__internal_event_filter)
        .with_test_writer()
        .try_init();
    }
  }
}

#[cfg(not(feature = "trace"))]
fn expand_tracing_init(_attribute_args: &AttributeArgs) -> Tokens {
  quote! {}
}
