use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq)]
pub struct Version {
  pub major : u32,
  pub minor : u32,
  pub patch : u32
}

impl Display for Version {
  fn fmt( &self, fmtr : &mut fmt::Formatter ) -> fmt::Result {
    write!( fmtr, "{}.{}.{}", self.major, self.minor, self.patch )
  }
}

impl FromStr for Version {
  type Err = String;

  fn from_str( s : &str ) -> Result<Version, Self::Err> {
    let parts : Vec<Result<u32, &str>> =
      s.split( '.' )
      .map( | elm | elm.parse::<u32>()
                             .map_err( |_| elm ) )
      .collect();

    if parts.len() != 3 {
      return
        Err( format!( "Invalid version format: expected 3 components, got {}."
           , parts.len() ) );
    }

    for part in &parts {
      match part {
        &Err( err ) =>
          return
            Err( format!( "Invalid version format: expected integer, got '{}'."
                         , err ) ),
        _ => {}
      }
    }

    Ok( Version {
      major: parts[0].unwrap(),
      minor: parts[1].unwrap(),
      patch: parts[2].unwrap()
    } )
  }
}

/// Gets the current version as a string.
#[macro_export]
macro_rules! version(
  () => ( env!( "CARGO_PKG_VERSION" ) )
);

#[test]
fn does_it_work() {
  let ver = FromStr::from_str( &version!() );
  assert_eq!( ver, Ok( Version { major: 2, minor: 0, patch: 1 } ) );

  let invalids = [ "nope", "1.0", "1.0.x", "1.x.0", "x.0.1", "x.x.x" ];

  for invalid in &invalids {
    let invv : Result<Version, String> = FromStr::from_str( invalid );
    assert!( invv.is_err() );
  }

  // Bad test is bad.
  assert_eq!( version!(), "2.0.1" );
}
