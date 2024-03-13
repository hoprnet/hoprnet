A simple crate for reducing the boilerplate when writing parsers with `syn`.

## Structs

```rust
#[derive(Clone, Debug, syn_derive::Parse, syn_derive::ToTokens)]
struct ExampleStruct {
	#[parse(Attribute::parse_outer)]
	#[to_tokens(|tokens, val| tokens.append_all(val))]
	attrs: Vec<Attribute>,

	path: Path,

	#[syn(parenthesized)]
	paren_token: Paren,

	#[syn(in = brace_token)]
	#[parse(Punctuated::parse_terminated)]
	args: Punctuated<Box<Expr>, Token![,]>,

	semi_token: Token![;],
}
```

`[syn(parenthesized)]`,
`[syn(braced)]`,
`[syn(bracketed)]`:
  Corresponds to the isonymous macros in `syn`.
  Must be attached to `Paren`, `Brace`, and `Bracket` fields, respectively.

`#[syn(in = Ident)]`:
  The field is read from inside the named delimiter pair.

`#[parse(fn(ParseStream) -> syn::Result<T>)]`:
  A function used to parse the field,
  often used with `Punctuated::parse_terminated`
  or `Attribute::parse_outer`.

`#[to_tokens(fn(&mut TokenStream, &T)]`:
  A function used to tokenize the field.
  Often used with `TokenStreamExt::append_all`,
  though for type resolution reasons this needs to be indirected through a closure expression.

## Enums

```rust
#[derive(Clone, Debug, syn_derive::Parse, syn_derive::ToTokens)]
enum ExampleEnum {
	#[parse(peek = Token![struct])]
	Struct(ItemStruct),
	#[parse(peek = Token![enum])]
	Enum(ItemEnum),

	Other {
		path: Path,
		semi_token: Token![;],
	}
}
```

`#[parse(prefix = fn(ParseStream) -> syn::Result<_>)]`:
  A prefix used for all branches, before doing the peeking.
  Useful when all branches support attributes, for example.
  The return value is ignored, which gives somewhat suboptimal performance, since the prefix is parsed twice.

`#[parse(peek = Token)]`:
  Checks whether the variant should be parsed.
  Even if multiple peeks succeed, only the first successful variant is attempted.

`#[parse(peek_func = fn(ParseStream) -> bool)]`:
  More powerful than `peek` (such as allowing `peek2`), but gives worse error messages on failure.
  `peek` should be preferred when possible.

# Feature flags
- `full` (enabled by default): enables `syn/full`, which is needed to parse complex expressions — such as closures — in attribute values.
  Without this, you can still use for example paths to functions, but this is much less convenient.

# Alternatives
- [`derive-syn-parse`](https://docs.rs/derive-syn-parse/latest/)
  does not handle `ToTokens`.
  It also seems to encourage throwing tokens away with its `prefix` and `postfix` attributes.
- [`parsel`](https://docs.rs/parsel/latest/)
  uses its own types for parentheses, meaning the AST types have different API from `syn`'s own.
