#![ cfg_attr( nightly, feature( doc_cfg ) ) ]
#![ doc = include_str!( "../README.md" ) ]

#![ doc    ( html_root_url = "https://docs.rs/pharos" ) ]
#![ deny   ( missing_docs                             ) ]
#![ forbid ( unsafe_code                              ) ]
#![ allow  ( clippy::suspicious_else_formatting       ) ]

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


mod error         ;
mod events        ;
mod observable    ;
mod pharos        ;
mod filter        ;
mod shared_pharos ;



pub use
{
	self::pharos :: { Pharos                                              } ,
	filter       :: { Filter                                              } ,
	observable   :: { Observable, ObservableLocal, ObserveConfig, Channel } ,
	events       :: { Events                                              } ,
	error        :: { PharErr, ErrorKind                                  } ,
	shared_pharos:: { SharedPharos                                        } ,
};


mod import
{
	pub(crate) use
	{
		std            :: { fmt, error::Error as ErrorTrait, ops::Deref, any::type_name  } ,
		std            :: { task::{ Poll, Context }, pin::Pin, future::Future, sync::Arc } ,
		futures        :: { Stream, Sink, SinkExt, ready, future::FutureExt, lock::Mutex } ,

		futures::channel::mpsc::
		{
			self                                      ,
			Sender            as FutSender            ,
			Receiver          as FutReceiver          ,
			UnboundedSender   as FutUnboundedSender   ,
			UnboundedReceiver as FutUnboundedReceiver ,
			SendError         as FutSendError         ,
		},
	};

	#[ cfg( test ) ]
	//
	pub(crate) use
	{
		assert_matches :: { assert_matches                      } ,
		futures        :: { future::poll_fn, executor::block_on } ,
	};
}

use import::*;


/// A pinned boxed future returned by the Observable::observe method.
//
pub type Observe<'a, Event, Error> = Pin<Box< dyn Future< Output = Result<Events<Event>, Error> > + 'a + Send >>;

/// A pinned boxed future returned by the ObservableLocal::observe_local method.
//
pub type ObserveLocal<'a, Event, Error> = Pin<Box< dyn Future< Output = Result<Events<Event>, Error> > + 'a >>;
