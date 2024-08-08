#![feature(test)]

extern crate test;
extern crate xmltree;

use test::Bencher;

use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use xmltree::Element;
use xmltree::ParseError;

fn _parse_file(filename: &str) -> std::result::Result<xmltree::Element, xmltree::ParseError> {
    let mut file = File::open(filename).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    Element::parse(data.as_bytes())
}

#[bench]
fn bench_01(b: &mut Bencher) {
    b.iter(|| {
        let filename = "tests/data/01.xml";
        let e: Element = _parse_file(filename).unwrap();

        assert_eq!(e.name, "project");
        let e2: &Element = e
            .get_child("libraries")
            .expect("Missing libraries child element");
        assert_eq!(e2.name, "libraries");

        assert!(e.get_child("doesnotexist").is_none());

        let mut buf = Vec::new();
        e.write(&mut buf).unwrap();

        let e2 = Element::parse(Cursor::new(buf)).unwrap();
        assert_eq!(e, e2);
    })
}

#[bench]
fn bench_02(b: &mut Bencher) {
    let filename = "tests/data/02.xml";
    b.iter(|| {
        let _ = _parse_file(filename).unwrap();
    });
}

#[bench]
fn bench_03(b: &mut Bencher) {
    let filename = "tests/data/03.xml";
    b.iter(|| {
        let _ = _parse_file(filename).unwrap();
    });
}

#[bench]
fn bench_04(b: &mut Bencher) {
    let filename = "tests/data/04.xml";
    b.iter(|| {
        let _ = _parse_file(filename).unwrap();
    });
}

#[bench]
fn bench_rw(b: &mut Bencher) {
    let filename = "tests/data/rw.xml";
    b.iter(|| {
        let e = _parse_file(filename).unwrap();

        let mut buf = Vec::new();
        e.write(&mut buf).unwrap();

        let e2 = Element::parse(Cursor::new(buf)).unwrap();
        assert_eq!(e, e2);
    })
}

#[bench]
fn bench_mal_01(b: &mut Bencher) {
    let data = r##"
        <?xml version="1.0" encoding="utf-8" standalone="yes"?>
        <names>
            <name first="bob" last="jones />
            <name first="elizabeth" last="smith" />
        </names>
    "##;

    b.iter(|| {
        let names_element = Element::parse(data.as_bytes());
        if let Err(ParseError::MalformedXml(..)) = names_element {
            // OK
        } else {
            panic!("unexpected parse result");
        }
    });
}

#[bench]
fn bench_mal_02(b: &mut Bencher) {
    let data = r##"
            this is not even close
            to XML
    "##;

    b.iter(|| {
        let names_element = Element::parse(data.as_bytes());
        if let Err(ParseError::MalformedXml(..)) = names_element {
            // OK
        } else {
            panic!("unexpected parse result");
        }
    });
}

#[bench]
fn bench_mal_03(b: &mut Bencher) {
    let data = r##"
        <?xml version="1.0" encoding="utf-8" standalone="yes"?>
        <names>
            <name first="bob" last="jones"></badtag>
            <name first="elizabeth" last="smith" />
        </names>
    "##;

    b.iter(|| {
        let names_element = Element::parse(data.as_bytes());
        if let Err(ParseError::MalformedXml(..)) = names_element {
            // OK
        } else {
            panic!("unexpected parse result");
        }
    });
}

#[bench]
fn bench_new(b: &mut Bencher) {
    b.iter(|| {
        let e = Element::new("foo");
        assert_eq!(e.name.as_str(), "foo");
        assert_eq!(e.attributes.len(), 0);
        assert_eq!(e.children.len(), 0);
        assert_eq!(e.get_text(), None);
    });
}

#[bench]
fn bench_take(b: &mut Bencher) {
    let data_xml_1 = r##"
        <?xml version="1.0" encoding="utf-8" standalone="yes"?>
        <names>
            <name first="bob" last="jones"></name>
            <name first="elizabeth" last="smith" />
            <remove_me key="value">
                <child />
            </remove_me>
        </names>
    "##;

    let data_xml_2 = r##"
        <?xml version="1.0" encoding="utf-8" standalone="yes"?>
        <names>
            <name first="bob" last="jones"></name>
            <name first="elizabeth" last="smith" />
        </names>
    "##;

    b.iter(|| {
        let mut data_1 = Element::parse(data_xml_1.as_bytes()).unwrap();
        let data_2 = Element::parse(data_xml_2.as_bytes()).unwrap();

        if let Some(removed) = data_1.take_child("remove_me") {
            assert_eq!(removed.children.len(), 1);
        } else {
            panic!("take_child failed");
        }

        assert_eq!(data_1, data_2);
    });
}

#[bench]
fn bench_ns1_rw(b: &mut Bencher) {
    let filename = "tests/data/ns2.xml";
    b.iter(|| {
        let e = _parse_file(filename).unwrap();
        let mut buf = Vec::new();
        e.write(&mut buf).unwrap();
    });
}

#[bench]
fn bench_ns2_rw(b: &mut Bencher) {
    let filename = "tests/data/ns2.xml";
    b.iter(|| {
        let e = _parse_file(filename).unwrap();
        let mut buf = Vec::new();
        e.write(&mut buf).unwrap();
    });
}
