use crate :: { import::* };

/// Predicate for filtering events.
///
/// This is an enum because closures that capture variables from
/// their environment need to be boxed. More often than not, an event will be a simple enum and
/// the predicate will just match on the variant, so it would be wasteful to impose boxing in those
/// cases, hence there is a function pointer variant which does not require boxing. This should
/// be preferred where possible.
///
/// ```
/// use pharos::*;
///
/// let a = 5;
///
/// // This closure captures the a variable from it's environment.
/// // We can still use it as a filter by boxing it with `closure`.
/// //
/// // Note: it depends on the circumstances, but often enough, we need to
/// // annotate the type of the event parameter to the predicate.
/// //
/// // For this example we use bool as event type for simplicity, but it works
/// // just the same if that's an enum.
/// //
/// let predicate = move |_: &bool| { a; true };
///
/// let filter = Filter::Closure( Box::new(predicate) );
///
/// // This one does not capture anything, so it can be stored as a function pointer
/// // without boxing.
/// //
/// let predicate = move |_: &bool| { true };
///
/// let filter = Filter::Pointer( predicate );
///
/// // You can also use actual functions as filters.
/// //
/// fn predicate_function( event: &bool ) -> bool { true }
///
/// let filter = Filter::Pointer( predicate_function );
/// ```
//
pub enum Filter<Event>

	where Event: Clone + 'static + Send ,

{
	/// A function pointer to a predicate to filter events.
	//
	Pointer( fn(&Event) -> bool ),

	/// A boxed closure to a predicate to filter events.
	//
	Closure( Box<dyn FnMut(&Event) -> bool + Send> ),
}


impl<Event> Filter<Event>  where Event: Clone + 'static + Send
{
	/// Invoke the predicate.
	//
	pub(crate) fn call( &mut self, evt: &Event ) -> bool
	{
		match self
		{
			Self::Pointer(f) => f(evt),
			Self::Closure(f) => f(evt),
		}
	}
}


impl<Event> fmt::Debug for Filter<Event>  where Event: Clone + 'static + Send
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		match self
		{
			Self::Pointer(_) => write!( f, "pharos::Filter<{}>::Pointer(_)", type_name::<Event>() ),
			Self::Closure(_) => write!( f, "pharos::Filter<{}>::Closure(_)", type_name::<Event>() ),
		}

	}
}



#[ cfg( test ) ]
//
mod tests
{
	use super::*;

	#[test]
	//
	fn debug()
	{
		let f = Filter::Pointer(           |b| *b   );
		let g = Filter::Closure( Box::new( |b| *b ) );

		assert_eq!( "pharos::Filter<bool>::Pointer(_)", &format!( "{:?}", f ) );
		assert_eq!( "pharos::Filter<bool>::Closure(_)", &format!( "{:?}", g ) );
	}
}
