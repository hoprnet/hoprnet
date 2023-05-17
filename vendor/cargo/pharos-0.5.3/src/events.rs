use crate :: { import::*, Filter, ObserveConfig, observable::Channel, PharErr, ErrorKind };


/// A stream of events. This is returned from [Observable::observe](crate::Observable::observe).
/// You will only start receiving events from the moment you call this. Any events in the observed
/// object emitted before will not be delivered.
///
/// For pharos 0.4.0 on x64 Linux: `std::mem::size_of::<Events<_>>() == 16`
//
#[ derive( Debug ) ]
//
pub struct Events<Event> where Event: Clone + 'static + Send
{
	rx: Receiver<Event>,
}


impl<Event> Events<Event> where Event: Clone + 'static + Send
{
	pub(crate) fn new( config: ObserveConfig<Event> ) -> (Self, Sender<Event>)
	{
		let (tx, rx) = match config.channel
		{
			Channel::Bounded( queue_size ) =>
			{
				let (tx, rx) = mpsc::channel( queue_size - 1 );

				( Sender::Bounded{ tx, filter: config.filter }, Receiver::Bounded{ rx } )
			}

			Channel::Unbounded =>
			{
				let (tx, rx) = mpsc::unbounded();

				( Sender::Unbounded{ tx, filter: config.filter }, Receiver::Unbounded{ rx } )
			}

			_ => unreachable!(),
		};


		( Self{ rx }, tx )
	}


	/// Disconnect from the observable object. This way the sender will stop sending new events
	/// and you can still continue to read any events that are still pending in the channel.
	//
	pub fn close( &mut self )
	{
		self.rx.close();
	}
}



// Just forward
//
impl<Event> Stream for Events<Event> where Event: Clone + 'static + Send
{
	type Item = Event;

	fn poll_next( mut self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll< Option<Self::Item> >
	{
		Pin::new( &mut self.rx ).poll_next( cx )
	}
}



/// The sender of the channel.
/// For pharos 0.4.0 on x64 Linux: `std::mem::size_of::<Sender<_>>() == 56`
//
pub(crate) enum Sender<Event> where Event: Clone + 'static + Send
{
	Bounded  { tx: FutSender         <Event>, filter: Option<Filter<Event>> } ,
	Unbounded{ tx: FutUnboundedSender<Event>, filter: Option<Filter<Event>> } ,
}




impl<Event> Sender<Event>  where Event: Clone + 'static + Send
{
	// Verify whether this observer is still around.
	//
	pub(crate) fn is_closed( &self ) -> bool
	{
		match self
		{
			Sender::Bounded  { tx, .. } => tx.is_closed(),
			Sender::Unbounded{ tx, .. } => tx.is_closed(),
		}
	}


	/// Check whether this sender is interested in this event.
	//
	pub(crate) fn filter( &mut self, evt: &Event ) -> bool
	{
		match self
		{
			Sender::Bounded  { filter, .. } => Self::filter_inner( filter, evt ),
			Sender::Unbounded{ filter, .. } => Self::filter_inner( filter, evt ),
		}
	}


	fn filter_inner( filter: &mut Option<Filter<Event>>, evt: &Event ) -> bool
	{
		match filter
		{
			Some(f) => f.call(evt),
			None    => true       ,
		}
	}
}



/// The receiver of the channel, abstracting over different channel types.
//
enum Receiver<Event> where Event: Clone + 'static + Send
{
	Bounded  { rx: FutReceiver<Event>          } ,
	Unbounded{ rx: FutUnboundedReceiver<Event> } ,
}


impl<Event> Receiver<Event> where Event: Clone + 'static + Send
{
	fn close( &mut self )
	{
		match self
		{
			Receiver::Bounded  { rx } => rx.close(),
			Receiver::Unbounded{ rx } => rx.close(),
		};
	}
}



impl<Event> fmt::Debug for Receiver<Event>  where Event: 'static + Clone + Send
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		match self
		{
			Self::Bounded  {..} => write!( f, "pharos::events::Receiver::<{}>::Bounded(_)"  , type_name::<Event>() ),
			Self::Unbounded{..} => write!( f, "pharos::events::Receiver::<{}>::Unbounded(_)", type_name::<Event>() ),
		}
	}
}




impl<Event> Stream for Receiver<Event> where Event: Clone + 'static + Send
{
	type Item = Event;

	fn poll_next( self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll< Option<Self::Item> >
	{
		match self.get_mut()
		{
			Receiver::Bounded  { rx } => Pin::new( rx ).poll_next( cx ),
			Receiver::Unbounded{ rx } => Pin::new( rx ).poll_next( cx ),
		}
	}
}



impl<Event> Sink<Event> for Sender<Event> where Event: Clone + 'static + Send
{
	type Error = PharErr;


	fn poll_ready( self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Result<(), Self::Error>>
	{
		match self.get_mut()
		{
			Sender::Bounded  { tx, .. } => Pin::new( tx ).poll_ready( cx ).map_err( Into::into ),
			Sender::Unbounded{ tx, .. } => Pin::new( tx ).poll_ready( cx ).map_err( Into::into ),
		}
	}


	fn start_send( self: Pin<&mut Self>, item: Event ) -> Result<(), Self::Error>
	{
		match self.get_mut()
		{
			Sender::Bounded  { tx, .. } => Pin::new( tx ).start_send( item ).map_err( Into::into ),
			Sender::Unbounded{ tx, .. } => Pin::new( tx ).start_send( item ).map_err( Into::into ),
		}
	}


	// Note that on futures-rs bounded channels poll_flush has a problematic implementation.
	// - it just calls poll_ready, which means it will be pending when the buffer is full. So
	//   it will make SinkExt::send hang, bad!
	// - it will swallow disconnected errors, so we don't get feedback allowing us to free slots.
	//
	// In principle channels are always flushed, because when the message is in the buffer, it's
	// ready for the reader to read. So this should just be a noop.
	//
	// We compensate for the error swallowing by checking `is_closed`.
	//
	fn poll_flush( self: Pin<&mut Self>, _cx: &mut Context<'_> ) -> Poll<Result<(), Self::Error>>
	{
		match self.get_mut()
		{
			Sender::Bounded  { tx, .. } =>
			{
				if tx.is_closed() { Poll::Ready(Err( ErrorKind::Closed.into() ))}
				else              { Poll::Ready(Ok ( ()                       ))}
			}

			Sender::Unbounded{ tx, .. } =>
			{
				if tx.is_closed() { Poll::Ready(Err( ErrorKind::Closed.into() ))}
				else              { Poll::Ready(Ok ( ()                       ))}
			}
		}
	}


	fn poll_close( self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Result<(), Self::Error>>
	{
		match self.get_mut()
		{
			Sender::Bounded  { tx, .. } => Pin::new( tx ).poll_close( cx ).map_err( Into::into ),
			Sender::Unbounded{ tx, .. } => Pin::new( tx ).poll_close( cx ).map_err( Into::into ),
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
		let e = Events::<bool>::new( ObserveConfig::default() );

		assert_eq!( "Events { rx: pharos::events::Receiver::<bool>::Unbounded(_) }", &format!( "{:?}", e.0 ) );
	}
}
