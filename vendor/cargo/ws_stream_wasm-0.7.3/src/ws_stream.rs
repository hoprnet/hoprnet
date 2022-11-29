use crate::{ import::*, * };


/// A futures 0.3 Sink/Stream of [WsMessage]. Created with [WsMeta::connect](crate::WsMeta::connect).
///
/// ## Closing the connection
///
/// When this is dropped, the connection closes, but you should favor calling one of the close
/// methods on [WsMeta](crate::WsMeta), which allow you to set a proper close code and reason.
///
/// Since this implements [`Sink`], it has to have a close method. This method will call the
/// web api [`WebSocket.close`](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)
/// without parameters. Eg. a default value of `1005` will be assumed for the close code. The
/// situation is the same when dropping without calling close.
///
/// **Warning**: This object holds the callbacks needed to receive events from the browser.
/// If you drop it before the close event was emitted, you will no longer receive events. Thus,
/// observers will never receive a `Close` event. Drop will issue a `Closing` event and this
/// will be the very last event observers receive. The the stream will end if `WsMeta` is also dropped.
///
/// See the [integration tests](https://github.com/najamelan/ws_stream_wasm/blob/release/tests/futures_codec.rs)
/// if you need an example.
///
//
pub struct WsStream
{
	ws: SendWrapper< Rc< WebSocket > >,

	// The queue of received messages
	//
	queue: SendWrapper< Rc<RefCell< VecDeque<WsMessage> >> >,

	// Last waker of task that wants to read incoming messages to be woken up on a new message
	//
	waker: SendWrapper< Rc<RefCell< Option<Waker> >> >,

	// Last waker of task that wants to write to the Sink
	//
	sink_waker: SendWrapper< Rc<RefCell< Option<Waker> >> >,

	// A pointer to the pharos of WsMeta for when we need to listen to events
	//
	pharos: SharedPharos<WsEvent>,

	// The callback closures.
	//
	_on_open : SendWrapper< Closure< dyn FnMut()               > >,
	_on_error: SendWrapper< Closure< dyn FnMut()               > >,
	_on_close: SendWrapper< Closure< dyn FnMut( JsCloseEvt   ) > >,
	_on_mesg : SendWrapper< Closure< dyn FnMut( MessageEvent ) > >,

	// This allows us to store a future to poll when Sink::poll_close is called
	//
	closer: Option<SendWrapper< Pin<Box< dyn Future< Output=() > + Send >> >>,
}


impl WsStream
{
	/// Create a new WsStream.
	//
	pub(crate) fn new
	(
		ws      : SendWrapper< Rc<WebSocket> > ,
		pharos  : SharedPharos<WsEvent>        ,
		on_open : SendWrapper< Closure< dyn FnMut()               > > ,
		on_error: SendWrapper< Closure< dyn FnMut()               > > ,
		on_close: SendWrapper< Closure< dyn FnMut( JsCloseEvt   ) > > ,

	) -> Self

	{
		let waker     : SendWrapper< Rc<RefCell<Option<Waker>>> > = SendWrapper::new( Rc::new( RefCell::new( None )) );
		let sink_waker: SendWrapper< Rc<RefCell<Option<Waker>>> > = SendWrapper::new( Rc::new( RefCell::new( None )) );

		let queue = SendWrapper::new( Rc::new( RefCell::new( VecDeque::new() ) ) );
		let q2    = queue.clone();
		let w2    = waker.clone();
		let ph2   = pharos.clone();


		// Send the incoming ws messages to the WsMeta object
		//
		#[ allow( trivial_casts ) ]
		//
		let on_mesg = Closure::wrap( Box::new( move |msg_evt: MessageEvent|
		{
			match WsMessage::try_from( msg_evt )
			{
				Ok (msg) => q2.borrow_mut().push_back( msg ),
				Err(err) => notify( ph2.clone(), WsEvent::WsErr( err ) ),
			}

			if let Some( w ) = w2.borrow_mut().take()
			{
				w.wake()
			}

		}) as Box< dyn FnMut( MessageEvent ) > );


		// Install callback
		//
		ws.set_onmessage  ( Some( on_mesg.as_ref().unchecked_ref() ) );


		// When the connection closes, we need to verify if there are any tasks
		// waiting on poll_next. We need to wake them up.
		//
		let ph    = pharos    .clone();
		let wake  = waker     .clone();
		let swake = sink_waker.clone();

		let wake_on_close = async move
		{
			let mut rx;

			// Scope to avoid borrowing across await point.
			//
			{
				match ph.observe_shared( Filter::Pointer( WsEvent::is_closed ).into() ).await
				{
					Ok(events) => rx = events               ,
					Err(e)     => unreachable!( "{:?}", e ) , // only happens if we closed it.
				}
			}

			rx.next().await;

			if let Some(w) = &*wake.borrow()
			{
				w.wake_by_ref();
			}

			if let Some(w) = &*swake.borrow()
			{
				w.wake_by_ref();
			}
		};

		spawn_local( wake_on_close );


		Self
		{
			ws                                      ,
			queue                                   ,
			waker                                   ,
			sink_waker                              ,
			pharos                                  ,
			closer    : None                        ,
			_on_mesg  : SendWrapper::new( on_mesg ) ,
			_on_open  : on_open                     ,
			_on_error : on_error                    ,
			_on_close : on_close                    ,
		}
	}



	/// Verify the [WsState] of the connection.
	//
	pub fn ready_state( &self ) -> WsState
	{
		self.ws.ready_state().try_into()

			// This can't throw unless the browser gives us an invalid ready state
			//
			.expect_throw( "Convert ready state from browser API" )
	}



	/// Access the wrapped [web_sys::WebSocket](https://docs.rs/web-sys/0.3.25/web_sys/struct.WebSocket.html) directly.
	///
	/// _ws_stream_wasm_ tries to expose all useful functionality through an idiomatic rust API, so hopefully
	/// you won't need this, however if I missed something, you can.
	///
	/// ## Caveats
	/// If you call `set_onopen`, `set_onerror`, `set_onmessage` or `set_onclose` on this, you will overwrite
	/// the event listeners from `ws_stream_wasm`, and things will break.
	//
	pub fn wrapped( &self ) -> &WebSocket
	{
		&self.ws
	}


	/// Wrap this object in [`IoStream`]. `IoStream` implements `AsyncRead`/`AsyncWrite`/`AsyncBufRead`.
	/// **Beware**: that this will transparenty include text messages as bytes.
	//
	pub fn into_io( self ) -> IoStream< WsStreamIo, Vec<u8> >
	{
		IoStream::new( WsStreamIo::new( self ) )
	}
}



impl fmt::Debug for WsStream
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		write!( f, "WsStream for connection: {}", self.ws.url() )
	}
}



impl Drop for WsStream
{
	// We don't block here, just tell the browser to close the connection and move on.
	//
	fn drop( &mut self )
	{
		match self.ready_state()
		{
			WsState::Closing | WsState::Closed => {}

			_ =>
			{
				// This can't fail. Only exceptions are related to invalid
				// close codes and reason strings to long.
				//
				self.ws.close().expect( "WsStream::drop - close ws socket" );


				// Notify Observers. This event is not emitted by the websocket API.
				//
				notify( self.pharos.clone(), WsEvent::Closing )
			}
		}

		self.ws.set_onmessage( None );
		self.ws.set_onerror  ( None );
		self.ws.set_onopen   ( None );
		self.ws.set_onclose  ( None );
	}
}



impl Stream for WsStream
{
	type Item = WsMessage;

	// Currently requires an unfortunate copy from Js memory to WASM memory. Hopefully one
	// day we will be able to receive the MessageEvt directly in WASM.
	//
	fn poll_next( self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Option< Self::Item >>
	{
		// Once the queue is empty, check the state of the connection.
		// When it is closing or closed, no more messages will arrive, so
		// return Poll::Ready( None )
		//
		if self.queue.borrow().is_empty()
		{
			*self.waker.borrow_mut() = Some( cx.waker().clone() );

			match self.ready_state()
			{
				WsState::Open | WsState::Connecting => Poll::Pending ,
				_                                   => None.into()   ,
			}
		}

		// As long as there is things in the queue, just keep reading
		//
		else { self.queue.borrow_mut().pop_front().into() }
	}
}



impl Sink<WsMessage> for WsStream
{
	type Error = WsErr;


	// Web API does not really seem to let us check for readiness, other than the connection state.
	//
	fn poll_ready( self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Result<(), Self::Error>>
	{
		match self.ready_state()
		{
			WsState::Connecting =>
			{
				*self.sink_waker.borrow_mut() = Some( cx.waker().clone() );

				Poll::Pending
			}

			WsState::Open => Ok(()).into(),
			_             => Err( WsErr::ConnectionNotOpen ).into(),
		}
	}


	fn start_send( self: Pin<&mut Self>, item: WsMessage ) -> Result<(), Self::Error>
	{
		match self.ready_state()
		{
			WsState::Open =>
			{
				// The send method can return 2 errors:
				// - unpaired surrogates in UTF (we shouldn't get those in rust strings)
				// - connection is already closed.
				//
				// So if this returns an error, we will return ConnectionNotOpen. In principle
				// we just checked that it's open, but this guarantees correctness.
				//
				match item
				{
					WsMessage::Binary( d ) => self.ws.send_with_u8_array( &d ).map_err( |_| WsErr::ConnectionNotOpen )? ,
					WsMessage::Text  ( s ) => self.ws.send_with_str     ( &s ).map_err( |_| WsErr::ConnectionNotOpen )? ,
				}

				Ok(())
			},


			// Connecting, Closing or Closed
			//
			_ => Err( WsErr::ConnectionNotOpen ),
		}
	}



	fn poll_flush( self: Pin<&mut Self>, _: &mut Context<'_> ) -> Poll<Result<(), Self::Error>>
	{
		Ok(()).into()
	}



	// TODO: find a simpler implementation, notably this needs to spawn a future.
	//       this can be done by creating a custom future. If we are going to implement
	//       events with pharos, that's probably a good time to re-evaluate this.
	//
	fn poll_close( mut self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Result<(), Self::Error>>
	{
		let state = self.ready_state();


		// First close the inner connection
		//
		if state == WsState::Connecting
		|| state == WsState::Open
		{
			// Can't fail
			//
			self.ws.close().unwrap_throw();

			notify( self.pharos.clone(), WsEvent::Closing );
		}


		// Check whether it's closed
		//
		match state
		{
			WsState::Closed => Ok(()).into(),

			_ =>
			{
				// Create a future that will resolve with the close event, so we can poll it.
				//
				if self.closer.is_none()
				{
					let mut ph = self.pharos.clone();

					let closer = async move
					{
						let mut rx = match ph.observe( Filter::Pointer( WsEvent::is_closed ).into() ).await
						{
							Ok(events) => events                    ,
							Err(e)     => unreachable!( "{:?}", e ) , // only happens if we closed it.
						};

						rx.next().await;
					};

					self.closer = Some(SendWrapper::new( closer.boxed() ));
				}


				let _ = ready!( self.closer.as_mut().unwrap().as_mut().poll(cx) );

				Ok(()).into()
			}
		}
	}
}




