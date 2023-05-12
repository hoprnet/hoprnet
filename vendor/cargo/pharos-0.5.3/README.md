# pharos

[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)
[![Build Status](https://github.com/najamelan/pharos/workflows/ci/badge.svg?branch=master)](https://github.com/najamelan/pharos/actions)
[![Docs](https://docs.rs/pharos/badge.svg)](https://docs.rs/pharos)
[![Crates.io](https://img.shields.io/crates/v/pharos)](https://crates.io/crates/pharos)
[![Crates.io downloads](https://img.shields.io/crates/d/pharos)](https://crates.io/crates/pharos)

> An introduction to pharos is available in many formats: [video](https://youtu.be/BAzsxW-nxh8), [wikipedia](https://en.wikipedia.org/wiki/Lighthouse_of_Alexandria) and it was even honored by many artists like [this painting by Micheal Turner](http://omeka.wustl.edu/omeka/files/original/2694d12580166e77d40afd37b492a78e.jpg).

More seriously, pharos is a small [observer](https://en.wikipedia.org/wiki/Observer_pattern) library that let's you create futures 0.3 streams that observers can listen to.

I created it to leverage interoperability we can create by using async [Stream](https://docs.rs/futures-preview/0.3.0-alpha.19/futures/stream/trait.Stream.html) and [Sink](https://docs.rs/futures-preview/0.3.0-alpha.19/futures/sink/trait.Sink.html) from the futures library. So you can use all stream combinators, forward it into Sinks and so on.

Minimal rustc version: 1.39.

## Table of Contents

- [Security](#security)
- [Limitations](#limitations)
- [Future work](#future-work)
- [Install](#install)
  - [Upgrade](#upgrade)
  - [Dependencies](#dependencies)
- [Usage](#usage)
  - [Filter](#filter)
- [API](#api)
- [Contributing](#contributing)
  - [Code of Conduct](#code-of-conduct)
- [License](#license)


## Security

The main issue with this crate right now is the possibility for the observable to outpace the observer. When using bounded channels, there is back pressure, which might allow DDOS attacks if using the pattern on arriving network packets. When using the unbounded channels, it might lead to excessive memory consumption if observers are outpaced.

TODO: To mitigate these problems effectively, I will add a ring channel where the channel will only buffer a certain amount events and will overwrite the oldest event instead of blocking the sender when the buffer is full.

This crate has: `#![ forbid( unsafe_code ) ]`, but it's dependency (futures library) uses a lot of unsafe code.


### Limitations

- only bounded and unbounded channel as back-end (for now)
- [`Events`] is not clonable right now (would require support from the channels we use as back-ends, eg. broadcast type channel)
- performance tweaking still needs to be done


### Future work

Please check out the [todo](https://github.com/najamelan/pharos/blob/master/TODO.md) for ambitions.


## Install

With [cargo add](https://github.com/killercup/cargo-edit):
`cargo add pharos`

With [cargo yaml](https://gitlab.com/storedbox/cargo-yaml):
```yaml
dependencies:

  pharos: ^0.5
```

With raw Cargo.toml
```toml
[dependencies]

   pharos = "0.5"
```

### Upgrade

Please check out the [changelog](https://github.com/najamelan/pharos/blob/master/CHANGELOG.md) when upgrading.


### Dependencies

This crate only has but one dependency. Cargo will automatically handle it for you. This dependency contains `unsafe` code.

```yaml
dependencies:

  futures: { version: ^0.3, default-features: false }
```

## Usage

`pharos` only works from async code, implementing Sink to notify observers. You can notify observers from within
`poll_*` methods by calling the poll methods of the [Sink](https://docs.rs/futures-preview/0.3.0-alpha.19/futures/sink/trait.Sink.html) impl directly. In async context you can use [SinkExt::send](https://docs.rs/futures-preview/0.3.0-alpha.19/futures/sink/trait.SinkExt.html#method.send). Observers must consume the messages fast enough, otherwise they will slow down the observable (bounded channel) or cause memory leak (unbounded channel).

Whenever observers want to unsubscribe, they can just drop the stream or call `close` on it. If you are an observable and you want to notify observers that no more messages will follow, just drop the pharos object. Failing that, create an event type that signifies EOF and send that to observers.

Your event type will be cloned once for each observer, so you might want to put it in an Arc if it's bigger than 2 pointer sizes (eg. there's no point putting an enum without data in an Arc).

When you need to notify a pharos object from several async tasks, you can use [`SharedPharos`]. This type allows observing and notifying with a shared reference and handles synchronyzation internally.

Examples can be found in the [examples](https://github.com/najamelan/pharos/tree/master/examples) directory. Here is the most basic one:

```rust
use
{
   pharos  :: { *                                      } ,
   futures :: { executor::block_on, StreamExt, SinkExt } ,
};


// here we put a pharos object on our struct
//
struct Goddess { pharos: Pharos<GoddessEvent> }


impl Goddess
{
   fn new() -> Self
   {
      Self { pharos: Pharos::default() }
   }

   // Send Goddess sailing so she can tweet about it!
   //
   pub async fn sail( &mut self )
   {
      // It's infallible. Observers that error will be dropped, since the only kind of errors on
      // channels are when the channel is closed.
      //
      self.pharos.send( GoddessEvent::Sailing ).await.expect( "notify observers" );
   }
}


// Event types need to implement clone, but you can wrap them in Arc if not. Also they will be
// cloned, so if you will have several observers and big event data, putting them in an Arc is
// definitely best. It has no benefit to put a simple dataless enum in an Arc though.
//
#[ derive( Clone, Debug, PartialEq, Copy ) ]
//
enum GoddessEvent
{
   Sailing
}


// This is the needed implementation of Observable. We might one day have a derive for this,
// but it's not so interesting, since you always have to point it to your pharos object,
// and when you want to be observable over several types of events, you might want to keep
// pharos in a hashmap over type_id, and a derive would quickly become a mess.
//
impl Observable<GoddessEvent> for Goddess
{
   type Error = PharErr;

   fn observe( &mut self, options: ObserveConfig<GoddessEvent>) -> Observe< '_, GoddessEvent, Self::Error >
   {
      self.pharos.observe( options )
   }
}


#[ async_std::main ]
//
async fn main()
{
  let mut isis = Goddess::new();

  // subscribe, the observe method takes options to let you choose:
  // - channel type (bounded/unbounded)
  // - a predicate to filter events
  //
  let mut events = isis.observe( Channel::Bounded( 3 ).into() ).await.expect( "observe" );

  // trigger an event
  //
  isis.sail().await;

  // read from stream and let's put on the console what the event looks like.
  //
  let evt = dbg!( events.next().await.unwrap() );

  // After this reads on the event stream will return None.
  //
  drop( isis );

  assert_eq!( GoddessEvent::Sailing, evt );
  assert_eq!( None, events.next().await );
}
```

### Filter

Sometimes you are not interested in all event types an observable can emit. A common use case is only listening for a
close event on a network connection. The observe method takes options which let you set the predicate. You can only
set one predicate for a given observer.

```rust
use pharos::*;

#[ derive( Clone, Debug, PartialEq, Copy ) ]
//
enum NetworkEvent
{
   Open    ,
   Error   ,
   Closing ,
   Closed  ,
}

struct Connection { pharos: Pharos<NetworkEvent> }

impl Observable<NetworkEvent> for Connection
{
   type Error = PharErr;

   fn observe( &mut self, options: ObserveConfig<NetworkEvent>) -> Observe< '_, NetworkEvent, Self::Error >
   {
       self.pharos.observe( options )
   }
}


#[ async_std::main ]
//
async fn main()
{
   let mut conn = Connection{ pharos: Pharos::default() };

   // We will only get close events. Note that here we don't need access to any surrounding variables in
   // the closure, so we can use a function pointer which avoids having to box the closure.
   //
   // Filter also has a variant `Closure` which allows you to pass in a `Box<dyn FnMut(&Event) -> bool + Send>`
   // if you need access to surrounding context to make the decision.
   //
   let filter = Filter::Pointer( |e| e == &NetworkEvent::Closed );

   // By creating the config object through into, other options will be defaults, notably here
   // this will use unbounded channels.
   //
   let observer = conn.observe( filter.into() ).await.expect( "observe" );

   // Combine both options.
   //
   let filter = Filter::Pointer( |e| e != &NetworkEvent::Closed );
   let opts   = ObserveConfig::from( filter ).channel( Channel::Bounded(5) );

   // Get everything but close events over a bounded channel with queue size 5.
   //
   let bounded_observer = conn.observe( opts ).await.expect( "observe" );
}
```


## API

API documentation can be found on [docs.rs](https://docs.rs/pharos).


## Contributing

Please check out the [contribution guidelines](https://github.com/najamelan/pharos/blob/master/CONTRIBUTING.md).


### Code of conduct

Any of the behaviors described in [point 4 "Unacceptable Behavior" of the Citizens Code of Conduct](http://citizencodeofconduct.org/#unacceptable-behavior) are not welcome here and might get you banned. If anyone including maintainers and moderators of the project fail to respect these/your limits, you are entitled to call them out.

## License

[Unlicence](https://unlicense.org/)
