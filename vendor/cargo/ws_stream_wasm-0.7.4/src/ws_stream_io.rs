use crate::{ import::*, WsErr, WsStream };


/// A wrapper around WsStream that converts errors into io::Error so that it can be
/// used for io (like `AsyncRead`/`AsyncWrite`).
///
/// You shouldn't need to use this manually. It is passed to [`IoStream`] when calling
/// [`WsStream::into_io`].
//
#[ derive(Debug) ]
//
pub struct WsStreamIo
{
	inner: WsStream
}



impl WsStreamIo
{
	/// Create a new WsStreamIo.
	//
	pub fn new( inner: WsStream ) -> Self
	{
		Self { inner }
	}
}



impl Stream for WsStreamIo
{
	type Item = Result< Vec<u8>, io::Error >;


	fn poll_next( mut self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Option< Self::Item >>
	{
		Pin::new( &mut self.inner ).poll_next( cx )

			.map( |opt|

				opt.map( |msg| Ok( msg.into() ) )
			)
	}
}



impl Sink< Vec<u8> > for WsStreamIo
{
	type Error = io::Error;


	fn poll_ready( mut self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Result<(), Self::Error>>
	{
		Pin::new( &mut self.inner ).poll_ready( cx ).map( convert_res_tuple )
	}


	fn start_send( mut self: Pin<&mut Self>, item: Vec<u8> ) -> Result<(), Self::Error>
	{
		Pin::new( &mut self.inner ).start_send( item.into() ).map_err( convert_err )
	}


	fn poll_flush( mut self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Result<(), Self::Error>>
	{
		Pin::new( &mut self.inner ).poll_flush( cx ).map( convert_res_tuple )
	}


	fn poll_close( mut self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Result<(), Self::Error>>
	{
		Pin::new( &mut self.inner ).poll_close( cx ).map( convert_res_tuple )
	}
}



fn convert_res_tuple( res: Result< (), WsErr> ) -> Result< (), io::Error >
{
	res.map_err( convert_err )
}



fn convert_err( err: WsErr ) -> io::Error
{
	match err
	{
		WsErr::ConnectionNotOpen => return io::Error::from( io::ErrorKind::NotConnected ) ,

		// This shouldn't happen, so panic for early detection.
		//
		_ => unreachable!(),
	}
}




