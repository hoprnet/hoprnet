#![cfg_attr( nightly, feature(doc_cfg) )]
#![cfg_attr( nightly, cfg_attr( nightly, doc = include_str!("../README.md") ))]
#![doc = ""] // empty doc line to handle missing doc warning when the feature is missing.

#![ doc    ( html_root_url = "https://docs.rs/ws_stream_wasm"            ) ]
#![ forbid ( unsafe_code                                                 ) ]
#![ allow  ( clippy::suspicious_else_formatting, clippy::needless_return ) ]


#![ warn
(
	missing_debug_implementations ,
	missing_docs                  ,
	nonstandard_style             ,
	rust_2018_idioms              ,
	trivial_casts                 ,
	trivial_numeric_casts         ,
	unused_extern_crates          ,
	unused_qualifications         ,
	single_use_lifetimes          ,
	unreachable_pub               ,
	variant_size_differences      ,
)]



mod error        ;
mod ws_event     ;
mod ws_message   ;
mod ws_meta      ;
mod ws_state     ;
mod ws_stream    ;
mod ws_stream_io ;

pub use
{
	error        :: { WsErr               } ,
	ws_event     :: { WsEvent, CloseEvent } ,
	ws_message   :: { WsMessage           } ,
	ws_meta      :: { WsMeta              } ,
	ws_state     :: { WsState             } ,
	ws_stream    :: { WsStream            } ,
	ws_stream_io :: { WsStreamIo          } ,
};



mod import
{
	pub(crate) use
	{
		futures              :: { prelude::{ Stream, Sink }, ready, StreamExt, FutureExt                         } ,
		std                  :: { io, collections::VecDeque, fmt, task::{ Context, Waker, Poll }, future::Future } ,
		std                  :: { rc::Rc, cell::{ RefCell }, pin::Pin, convert::{ TryFrom, TryInto }             } ,
		js_sys               :: { ArrayBuffer, Uint8Array                                                        } ,
		wasm_bindgen         :: { closure::Closure, JsCast, JsValue, UnwrapThrowExt                              } ,
		web_sys              :: { *, BinaryType, Blob, WebSocket, CloseEvent as JsCloseEvt, DomException         } ,
		js_sys               :: { Array                                                                          } ,
		pharos               :: { SharedPharos, PharErr, Observable, Observe, Filter, ObserveConfig,             } ,
		wasm_bindgen_futures :: { spawn_local                                                                    } ,
		async_io_stream      :: { IoStream                                                                       } ,
		thiserror            :: { Error                                                                          } ,
		send_wrapper         :: { SendWrapper                                                                    } ,
	};
}


use import::*;

/// Helper function to reduce code bloat
//
pub(crate) fn notify( pharos: SharedPharos<WsEvent>, evt: WsEvent )
{
	let notify = async move
	{
		pharos.notify( evt ).await

			.map_err( |e| unreachable!( "{:?}", e ) ).unwrap(); // only happens if we closed it.
	};

	spawn_local( notify );
}
