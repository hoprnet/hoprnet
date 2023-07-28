/*
 * SPDX-FileCopyrightText: Oliver Tale-Yazdi <oliver@tasty.limo>
 * SPDX-License-Identifier: (GPL-3.0 or Apache-2.0)
 */

#![doc = include_str!(env!("README_PATH"))]
#![deny(unsafe_code)]

use core::ops::Deref;
use proc_macro2::Span;
use quote::{quote_spanned, ToTokens};

mod test;

/// Creates a compile-time warning for proc macro use. See [DeprecatedWarningBuilder] for usage.
#[derive(Clone)]
pub enum Warning {
	/// A *deprecation* warning that notifies users of outdated types and functions.
	Deprecated {
		name: String,
		index: Option<usize>,
		message: String,
		links: Vec<String>,
		span: Span,
	},
}

/// A compile-time warning that was already subject to formatting.
///
/// Any content will be pasted as-is.
#[derive(Clone)]
pub enum FormattedWarning {
	/// A *deprecation* warning.
	Deprecated {
		/// Unique name of this warning.
		///
		/// Must be unique in the case that multiple of these warnings are emitted, for example by
		/// appending a counter.
		name: syn::Ident,
		/// The exact note to be used for `note = ""`.
		note: String,
		/// The span of the warning.
		///
		/// Should be set to the original location of where the warning should be emitted.
		span: Option<Span>,
	},
}

impl FormattedWarning {
	/// Create a new deprecated warning that already was formatted by the caller.
	#[must_use]
	pub fn new_deprecated<'a, S, T>(name: S, note: T, span: Span) -> Self
	where
		S: Into<&'a str>,
		T: Into<String>,
	{
		Self::Deprecated {
			name: syn::Ident::new(name.into(), span),
			note: note.into(),
			span: Some(span),
		}
	}
}

/// Gradually build a *deprecation* `Warning`.
///
/// # Example
///
/// ```rust
/// use proc_macro_warning::Warning;
///
/// let warning = Warning::new_deprecated("my_macro")
///     .old("my_macro()")
///     .new("my_macro::new()")
///     .help_link("https:://example.com")
///     // Normally you use the input span, but this is an example:
///     .span(proc_macro2::Span::call_site())
///     .build();
///
/// let mut warnings = vec![warning];
/// // When adding more, you will need to build each with `.index`.
///
/// // In a proc macro you can expand them in a private module:
/// quote::quote! {
///     mod warnings {
///         #(
///             #warnings
///         )*
///     }
/// };
/// ```
#[derive(Default)]
pub struct DeprecatedWarningBuilder {
	title: String,
	index: Option<usize>,
	old: Option<String>,
	new: Option<String>,
	links: Vec<String>,
	span: Option<Span>,
}

impl DeprecatedWarningBuilder {
	/// Create a new *deprecated* warning builder with the given title.
	///
	/// The title must be unique for each warning.
	#[must_use]
	pub fn from_title(title: &str) -> DeprecatedWarningBuilder {
		DeprecatedWarningBuilder { title: title.into(), ..Default::default() }
	}

	/// Set an optional index in case that a warning appears multiple times.
	///
	/// Must be set if a warning appears multiple times.
	#[must_use]
	pub fn index(self, index: usize) -> DeprecatedWarningBuilder {
		DeprecatedWarningBuilder { index: Some(index), ..self }
	}

	/// The old *deprecated* way of doing something.
	///
	/// Should complete the sentence "It is deprecated to ...".
	#[must_use]
	pub fn old(self, old: &str) -> DeprecatedWarningBuilder {
		DeprecatedWarningBuilder { old: Some(old.into()), ..self }
	}

	/// The *new* way of doing something.
	///
	/// Should complete the sentence "Please instead ...".
	#[must_use]
	pub fn new(self, new: &str) -> DeprecatedWarningBuilder {
		DeprecatedWarningBuilder { new: Some(new.into()), ..self }
	}

	/// A help link for the user to explain the transition and justification.
	#[must_use]
	pub fn help_link(self, link: &str) -> DeprecatedWarningBuilder {
		DeprecatedWarningBuilder { links: vec![link.into()], ..self }
	}

	/// Multiple help links for the user to explain the transition and justification.
	#[must_use]
	pub fn help_links(self, links: &[&str]) -> DeprecatedWarningBuilder {
		DeprecatedWarningBuilder { links: links.iter().map(|s| s.deref().into()).collect(), ..self }
	}

	/// The span of the warning.
	#[must_use]
	pub fn span(self, span: Span) -> DeprecatedWarningBuilder {
		DeprecatedWarningBuilder { span: Some(span), ..self }
	}

	/// Build the warning.
	pub fn maybe_build(self) -> Result<Warning, String> {
		let span = self.span.unwrap_or_else(Span::call_site);
		let title = self.title;
		let old = self.old.ok_or("Missing old")?;
		let new = self.new.ok_or("Missing new")?;
		let message = format!("It is deprecated to {}.\nPlease instead {}.", old, new);

		Ok(Warning::Deprecated { name: title, index: self.index, message, links: self.links, span })
	}

	/// Unwraps [`Self::maybe_build`] for convenience.
	#[must_use]
	pub fn build(self) -> Warning {
		self.maybe_build().expect("maybe_build failed")
	}
}

impl Warning {
	/// Create a new *deprecated* warning.
	#[must_use]
	pub fn new_deprecated(title: &str) -> DeprecatedWarningBuilder {
		DeprecatedWarningBuilder::from_title(title)
	}

	/// Sanitize the warning message.
	fn final_deprecated_message(&self) -> String {
		let (message, links) = match self {
			Warning::Deprecated { message, links, .. } => (message, links),
		};

		let lines = message.trim().lines().map(|line| line.trim_start());
		// Prepend two tabs to each line
		let message = lines.map(|line| format!("\t\t{}", line)).collect::<Vec<_>>().join("\n");

		if !links.is_empty() {
			let link =
				links.iter().map(|l| format!("<{}>", l)).collect::<Vec<_>>().join("\n\t\t\t");
			format!("\n{}\n\n\t\tFor more info see:\n\t\t\t{}", message, link)
		} else {
			format!("\n{}", message)
		}
	}

	/// Sanitize the warning name.
	fn final_deprecated_name(&self) -> syn::Ident {
		let (index, name, span) = match self {
			Warning::Deprecated { index, name, span, .. } => (*index, name, *span),
		};

		let name = match index {
			Some(i) => format!("{}_{}", name, i),
			None => name.clone(),
		};
		syn::Ident::new(&name, span)
	}
}

impl From<Warning> for FormattedWarning {
	fn from(val: Warning) -> Self {
		match val {
			Warning::Deprecated { span, .. } => FormattedWarning::Deprecated {
				name: val.final_deprecated_name(),
				note: val.final_deprecated_message(),
				span: Some(span),
			},
		}
	}
}

impl ToTokens for Warning {
	fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
		let formatted: FormattedWarning = self.clone().into();
		formatted.to_tokens(stream);
	}
}

impl ToTokens for FormattedWarning {
	fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
		let (name, note, span) = match self {
			FormattedWarning::Deprecated { name, note, span } => (name, note, span),
		};
		let span = span.unwrap_or_else(Span::call_site);

		let q = quote_spanned!(span =>
			/// This function should not be called and only exists to emit a compiler warning.
			///
			/// It is a No-OP in any case.
			#[allow(dead_code)]
			#[allow(non_camel_case_types)]
			#[allow(non_snake_case)]
			fn #name() {
				#[deprecated(note = #note)]
				#[allow(non_upper_case_globals)]
				const _w: () = ();
				let _ = _w;
			}
		);
		q.to_tokens(stream);
	}
}
