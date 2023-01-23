use crate::{ import::*, WsErr, WsState, WsStream, WsEvent, CloseEvent, notify };


/// The meta data related to a websocket. Allows access to the methods on the WebSocket API.
/// This is split from the `Stream`/`Sink` so you can pass the latter to a combinator whilst
/// continuing to use this API.
///
/// A `WsMeta` instance is observable through the [`pharos::Observable`](https://docs.rs/pharos/0.4.3/pharos/trait.Observable.html)
/// trait. The type of event is [WsEvent]. In the case of a Close event, there will be additional information included
/// as a [CloseEvent].
///
/// When you drop this, the connection does not get closed, however when you drop [WsStream] it does.
///
/// Most of the methods on this type directly map to the web API. For more documentation, check the
/// [MDN WebSocket documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket).
//
pub struct WsMeta
{
	ws    : SendWrapper< Rc<WebSocket> > ,
	pharos: SharedPharos<WsEvent>        ,
}



impl WsMeta
{
	const OPEN_CLOSE: Filter<WsEvent> = Filter::Pointer( |evt: &WsEvent| evt.is_open() | evt.is_closed() );

	/// Connect to the server. The future will resolve when the connection has been established with a successful WebSocket
	/// handshake.
	///
	/// This returns both a [WsMeta] (allow manipulating and requesting meta data for the connection) and
	/// a [WsStream] (`Stream`/`Sink` over [WsMessage](crate::WsMessage)). [WsStream] can be wrapped to obtain
	/// `AsyncRead`/`AsyncWrite`/`AsyncBufRead` with [WsStream::into_io].
	///
	/// ## Errors
	///
	/// Browsers will forbid making websocket connections to certain ports. See this [Stack Overflow question](https://stackoverflow.com/questions/4313403/why-do-browsers-block-some-ports/4314070).
	/// `connect` will return a [WsErr::ConnectionFailed] as it is indistinguishable from other connection failures.
	///
	/// If the URL is invalid, a [WsErr::InvalidUrl] is returned. See the [HTML Living Standard](https://html.spec.whatwg.org/multipage/web-sockets.html#dom-websocket) for more information.
	///
	/// When the connection fails (server port not open, wrong ip, wss:// on ws:// server, ... See the [HTML Living Standard](https://html.spec.whatwg.org/multipage/web-sockets.html#dom-websocket)
	/// for details on all failure possibilities), a [WsErr::ConnectionFailed] is returned.
	///
	/// **Note**: Sending protocols to a server that doesn't support them will make the connection fail.
	//
	pub async fn connect( url: impl AsRef<str>, protocols: impl Into<Option<Vec<&str>>> )

		-> Result< (Self, WsStream), WsErr >
	{
		let res = match protocols.into()
		{
			None => WebSocket::new( url.as_ref() ),

			Some(v) =>
			{
				let js_protos = v.iter().fold( Array::new(), |acc, proto|
				{
					acc.push( &JsValue::from_str( proto ) );
					acc
				});

				WebSocket::new_with_str_sequence( url.as_ref(), &js_protos )
			}
		};


		// Deal with errors from the WebSocket constructor.
		//
		let ws = match res
		{
			Ok(ws) => SendWrapper::new( Rc::new( ws ) ),

			Err(e) =>
			{
				let de: &DomException = e.unchecked_ref();

				match de.code()
				{
					DomException::SYNTAX_ERR =>

						return Err( WsErr::InvalidUrl{ supplied: url.as_ref().to_string() } ),


					_ => unreachable!(),
				};
			}
		};


		// Create our pharos.
		//
		let mut pharos = SharedPharos::default();
		let ph1        = pharos.clone();
		let ph2        = pharos.clone();
		let ph3        = pharos.clone();
		let ph4        = pharos.clone();


		// Setup our event listeners
		//
		#[ allow( trivial_casts ) ]
		//
		let on_open = Closure::wrap( Box::new( move ||
		{
			// notify observers
			//
			notify( ph1.clone(), WsEvent::Open )


		}) as Box< dyn FnMut() > );


		// TODO: is there no information at all in an error?
		//
		#[ allow( trivial_casts ) ]
		//
		let on_error = Closure::wrap( Box::new( move ||
		{
			// notify observers.
			//
			notify( ph2.clone(), WsEvent::Error )

		}) as Box< dyn FnMut() > );


		#[ allow( trivial_casts ) ]
		//
		let on_close = Closure::wrap( Box::new( move |evt: JsCloseEvt|
		{
			let c = WsEvent::Closed( CloseEvent
			{
				code     : evt.code()     ,
				reason   : evt.reason()   ,
				was_clean: evt.was_clean(),
			});

			notify( ph3.clone(), c )

		}) as Box< dyn FnMut( JsCloseEvt ) > );


		ws.set_onopen ( Some( on_open .as_ref().unchecked_ref() ));
		ws.set_onclose( Some( on_close.as_ref().unchecked_ref() ));
		ws.set_onerror( Some( on_error.as_ref().unchecked_ref() ));



		// Listen to the events to figure out whether the connection opens successfully. We don't want to deal with
		// the error event. Either a close event happens, in which case we want to recover the CloseEvent to return it
		// to the user, or an Open event happens in which case we are happy campers.
		//
		let mut evts = pharos.observe( Self::OPEN_CLOSE.into() ).await

			.expect( "we didn't close pharos" )
		;

		// If the connection is closed, return error
		//
		if let Some( WsEvent::Closed(evt) ) = evts.next().await
		{
			return Err( WsErr::ConnectionFailed{ event: evt } )
		}


		// We don't handle Blob's
		//
		ws.set_binary_type( BinaryType::Arraybuffer );


		Ok
		((
			Self
			{
				pharos,
				ws: ws.clone(),
			},

			WsStream::new
			(
				ws,
				ph4,
				SendWrapper::new( on_open  ),
				SendWrapper::new( on_error ),
				SendWrapper::new( on_close ),
			)
		))
	}



	/// Close the socket. The future will resolve once the socket's state has become `WsState::CLOSED`.
	/// See: [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)
	//
	pub async fn close( &self ) -> Result< CloseEvent, WsErr >
	{
		match self.ready_state()
		{
			WsState::Closed  => return Err( WsErr::ConnectionNotOpen ),
			WsState::Closing => {}

			_ =>
			{
				// This can not throw normally, because the only errors the API can return is if we use a code or
				// a reason string, which we don't.
				// See [MDN](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close#Exceptions_thrown).
				//
				self.ws.close().unwrap_throw();


				// Notify Observers
				//
				notify( self.pharos.clone(), WsEvent::Closing )
			}
		}


		let mut evts = match self.pharos.observe_shared( Filter::Pointer( WsEvent::is_closed ).into() ).await
		{
			Ok(events) => events                    ,
			Err(e)     => unreachable!( "{:?}", e ) , // only happens if we closed it.
		};

		// We promised the user a CloseEvent, so we don't have much choice but to unwrap this. In any case, the stream will
		// never end and this will hang if the browser fails to send a close event.
		//
		let ce = evts.next().await.expect_throw( "receive a close event" );

		if let WsEvent::Closed(e) = ce { Ok( e )        }
		else                          { unreachable!() }
	}




	/// Close the socket. The future will resolve once the socket's state has become `WsState::CLOSED`.
	/// See: [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)
	//
	pub async fn close_code( &self, code: u16  ) -> Result<CloseEvent, WsErr>
	{
		match self.ready_state()
		{
			WsState::Closed  => return Err( WsErr::ConnectionNotOpen ),
			WsState::Closing => {}

			_ =>
			{
				match self.ws.close_with_code( code )
				{
					// Notify Observers
					//
					Ok(_) => notify( self.pharos.clone(), WsEvent::Closing ),


					Err(_) =>
					{
						return Err( WsErr::InvalidCloseCode{ supplied: code } );
					}
				}
			}
		}


		let mut evts = match self.pharos.observe_shared( Filter::Pointer( WsEvent::is_closed ).into() ).await
		{
			Ok(events) => events                    ,
			Err(e)     => unreachable!( "{:?}", e ) , // only happens if we closed it.
		};

		let ce = evts.next().await.expect_throw( "receive a close event" );

		if let WsEvent::Closed(e) = ce { Ok(e)          }
		else                          { unreachable!() }
	}



	/// Close the socket. The future will resolve once the socket's state has become `WsState::CLOSED`.
	/// See: [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)
	//
	pub async fn close_reason( &self, code: u16, reason: impl AsRef<str>  ) -> Result<CloseEvent, WsErr>
	{
		match self.ready_state()
		{
			WsState::Closed  => return Err( WsErr::ConnectionNotOpen ),
			WsState::Closing => {}

			_ =>
			{
				if reason.as_ref().len() > 123
				{
					return Err( WsErr::ReasonStringToLong );
				}


				match self.ws.close_with_code_and_reason( code, reason.as_ref() )
				{
					// Notify Observers
					//
					Ok(_) => notify( self.pharos.clone(), WsEvent::Closing ),


					Err(_) =>
					{
						return Err( WsErr::InvalidCloseCode{ supplied: code } )
					}
				}
			}
		}

		let mut evts = match self.pharos.observe_shared( Filter::Pointer( WsEvent::is_closed ).into() ).await
		{
			Ok(events) => events                    ,
			Err(e)     => unreachable!( "{:?}", e ) , // only happens if we closed it.
		};

		let ce = evts.next().await.expect_throw( "receive a close event" );

		if let WsEvent::Closed(e) = ce { Ok(e)          }
		else                           { unreachable!() }
	}



	/// Verify the [WsState] of the connection.
	//
	pub fn ready_state( &self ) -> WsState
	{
		self.ws.ready_state().try_into()

			// This can't throw unless the browser gives us an invalid ready state.
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


	/// The number of bytes of data that have been queued but not yet transmitted to the network.
	///
	/// **NOTE:** that this is the number of bytes buffered by the underlying platform WebSocket
	/// implementation. It does not reflect any buffering performed by _ws_stream_wasm_.
	//
	pub fn buffered_amount( &self ) -> u32
	{
		self.ws.buffered_amount()
	}


	/// The extensions selected by the server as negotiated during the connection.
	///
	/// **NOTE**: This is an untested feature. The back-end server we use for testing (_tungstenite_)
	/// does not support Extensions.
	//
	pub fn extensions( &self ) -> String
	{
		self.ws.extensions()
	}


	/// The name of the sub-protocol the server selected during the connection.
	///
	/// This will be one of the strings specified in the protocols parameter when
	/// creating this WsMeta instance.
	//
	pub fn protocol(&self) -> String
	{
		self.ws.protocol()
	}


	/// Retrieve the address to which this socket is connected.
	//
	pub fn url( &self ) -> String
	{
		self.ws.url()
	}
}



impl fmt::Debug for WsMeta
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		write!( f, "WsMeta for connection: {}", self.url() )
	}
}



impl Observable<WsEvent> for WsMeta
{
	type Error = PharErr;

	fn observe( &mut self, options: ObserveConfig<WsEvent> ) -> Observe< '_, WsEvent, Self::Error >
	{
		self.pharos.observe( options )
	}
}


