//! Crate specific errors.
//
use crate::{ import::*, CloseEvent };


/// The error type for errors happening in `ws_stream_wasm`.
///
//
#[ derive( Debug, Error, Clone, PartialEq, Eq ) ] #[ non_exhaustive ]
//
pub enum WsErr
{
	/// Invalid input to [WsState::try_from( u16 )](crate::WsState).
	//
	#[ error( "Invalid input to conversion to WsReadyState: {supplied}" ) ]
	//
	InvalidWsState
	{
		/// The user supplied value that is invalid.
		//
		supplied: u16
	},

	/// When trying to send and [WsState](crate::WsState) is anything but [WsState::Open](crate::WsState::Open) this error is returned.
	//
	#[ error( "The connection state is not \"Open\"." ) ]
	//
	ConnectionNotOpen,

	/// An invalid URL was given to [WsMeta::connect](crate::WsMeta::connect), please see:
	/// [HTML Living Standard](https://html.spec.whatwg.org/multipage/web-sockets.html#dom-websocket).
	//
	#[ error( "An invalid URL was given to the connect method: {supplied}" ) ]
	//
	InvalidUrl
	{
		/// The user supplied value that is invalid.
		//
		supplied: String
	},

	/// An invalid close code was given to a close method. For valid close codes, please see:
	/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CloseEvent#Status_codes).
	//
	#[ error( "An invalid close code was given to a close method: {supplied}" ) ]
	//
	InvalidCloseCode
	{
		/// The user supplied value that is invalid.
		//
		supplied: u16
	},


	/// The reason string given to a close method is longer than 123 bytes, please see:
	/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close).
	//
	#[ error( "The reason string given to a close method is to long." ) ]
	//
	ReasonStringToLong,


	/// Failed to connect to the server.
	//
	#[ error( "Failed to connect to the server. CloseEvent: {event:?}" ) ]
	//
	ConnectionFailed
	{
		/// The close event that might hold extra code and reason information.
		//
		event: CloseEvent
	},


	/// When converting the JavaScript Message into a WsMessage, it's possible that
	/// a String message doesn't convert correctly as Js does not guarantee that
	/// strings are valid Unicode. Happens in `impl TryFrom< MessageEvent > for WsMessage`.
	//
	#[ error( "Received a String message that couldn't be decoded to valid UTF-8" ) ]
	//
	InvalidEncoding,


	/// When converting the JavaScript Message into a WsMessage, it's not possible to
	/// convert Blob type messages, as Blob is a streaming type, that needs to be read
	/// asynchronously. If you are using the type without setting up the connection with
	/// [`WsMeta::connect`](crate::WsMeta::connect), you have to make sure to set the binary
	/// type of the connection to `ArrayBuffer`.
	///
	/// Happens in `impl TryFrom< MessageEvent > for WsMessage`.
	//
	#[ error( "Received a Blob message that couldn't converted." ) ]
	//
	CantDecodeBlob,


	/// When converting the JavaScript Message into a WsMessage, the data type was neither
	/// `Arraybuffer`, `String` nor `Blob`. This should never happen. If it does, please
	/// try to make a reproducible example and file an issue.
	///
	/// Happens in `impl TryFrom< MessageEvent > for WsMessage`.
	//
	#[ error( "Received a message that is neither ArrayBuffer, String or Blob." ) ]
	//
	UnknownDataType,
}


