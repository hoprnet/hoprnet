use core::marker::PhantomData;
use crate::core::{ enc, dec };


pub struct RawValue<'de>(&'de [u8]);

struct RawValueReader<'r, 'de, R>
where R: dec::Read<'de>
{
    reader: &'r mut R,
    readn: usize,
    _phantom: PhantomData<&'de [u8]>
}

impl<'r, 'de, R> RawValueReader<'r, 'de, R>
where R: dec::Read<'de>
{
    #[inline]
    fn new(reader: &'r mut R) -> RawValueReader<'r, 'de, R> {
        RawValueReader {
            reader,
            readn: 0,
            _phantom: PhantomData
        }
    }
}

impl<'r, 'de, R> dec::Read<'de> for RawValueReader<'r, 'de, R>
where R: dec::Read<'de>
{
    type Error = R::Error;

    #[inline]
    fn fill<'short>(&'short mut self, want: usize) -> Result<dec::Reference<'de, 'short>, Self::Error> {
        let want = match self.readn.checked_add(want) {
            Some(n) => n,
            None => return Ok(dec::Reference::Long(&[]))
        };

        let buf = match self.reader.fill(want)? {
            dec::Reference::Long(buf)
                if buf.len() >= self.readn => dec::Reference::Long(&buf[self.readn..]),
            dec::Reference::Long(_) => dec::Reference::Long(&[]),
            dec::Reference::Short(buf) => dec::Reference::Short(buf)
        };

        Ok(buf)
    }

    #[inline]
    fn advance(&mut self, n: usize) {
        self.readn += n;
    }

    #[inline]
    fn step_in(&mut self) -> bool {
        self.reader.step_in()
    }

    #[inline]
    fn step_out(&mut self) {
        self.reader.step_out()
    }
}

impl<'de> dec::Decode<'de> for RawValue<'de> {
    #[inline]
    fn decode<R: dec::Read<'de>>(reader: &mut R) -> Result<Self, dec::Error<R::Error>> {
        let name = &"raw-value";

        let mut reader = RawValueReader::new(reader);
        let _ignore = dec::IgnoredAny::decode(&mut reader)?;

        let buf = match reader.reader.fill(reader.readn).map_err(dec::Error::Read)? {
            dec::Reference::Long(buf)
                if buf.len() >= reader.readn => &buf[..reader.readn],
            dec::Reference::Long(buf) => return Err(dec::Error::require_length(name, Some(buf.len()))),
            dec::Reference::Short(_) => return Err(dec::Error::require_borrowed(name))
        };

        reader.reader.advance(reader.readn);

        Ok(RawValue(buf))
    }
}

impl<'de> enc::Encode for RawValue<'de> {
    #[inline]
    fn encode<W: enc::Write>(&self, writer: &mut W) -> Result<(), enc::Error<W::Error>> {
        writer.push(self.0).map_err(enc::Error::Write)
    }
}

#[test]
#[cfg(feature = "use_std")]
fn test_raw_value() {
    use crate::core::enc::Encode;
    use crate::core::dec::Decode;
    use crate::core::utils::{ BufWriter, SliceReader };
    use crate::core::types;

    let buf = {
        let mut buf = BufWriter::new(Vec::new());

        types::Map(&[
            ("bar", types::Map(&[
                ("value", 0x99u32)
            ][..]))
        ][..]).encode(&mut buf).unwrap();

        buf
    };

    let mut reader = SliceReader::new(buf.buffer());
    let map = <types::Map<Vec<(&str, RawValue<'_>)>>>::decode(&mut reader).unwrap();

    assert_eq!(map.0.len(), 1);
    assert_eq!(map.0[0].0, "bar");

    let bar_raw_value = &map.0[0].1;

    let buf2 = {
        let mut buf = BufWriter::new(Vec::new());

        types::Map(&[
            ("bar", bar_raw_value)
        ][..]).encode(&mut buf).unwrap();

        buf
    };

    assert_eq!(buf.buffer(), buf2.buffer());

    type Bar<'a> = types::Map<Vec<(&'a str, u32)>>;

    let mut reader = SliceReader::new(buf2.buffer());
    let map2 = <types::Map<Vec<(&str, Bar)>>>::decode(&mut reader).unwrap();

    assert_eq!(map2.0.len(), 1);
    assert_eq!(map2.0[0].0, "bar");

    let bar = &map2.0[0].1;

    assert_eq!(bar.0.len(), 1);
    assert_eq!(bar.0[0].0, "value");
    assert_eq!(bar.0[0].1, 0x99);
}
