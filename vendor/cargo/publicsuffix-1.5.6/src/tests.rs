use crate::{errors::ErrorKind, request, List};
use rspec::describe;

lazy_static::lazy_static! {
    static ref LIST: List = List::fetch().unwrap();
}

#[test]
fn list_behaviour() {
    rspec::run(&describe("the list", (), |ctx| {
        ctx.it("should not be empty", |_| {
            assert!(!LIST.all().is_empty());
        });

        ctx.it("should have ICANN domains", |_| {
            assert!(!LIST.icann().is_empty());
        });

        ctx.it("should have private domains", |_| {
            assert!(!LIST.private().is_empty());
        });

        ctx.it("should have at least 1000 domains", |_| {
            assert!(LIST.all().len() > 1000);
        });
    }));

    rspec::run(&describe("the official test", (), |_| {
        let tests = "https://raw.githubusercontent.com/publicsuffix/list/master/tests/tests.txt";
        let body = request(tests).unwrap();

        let mut parse = false;

        for (i, line) in body.lines().enumerate() {
            match line {
                line if line.trim().is_empty() => {
                    parse = true;
                    continue;
                }
                line if line.starts_with("//") => {
                    continue;
                }
                line => {
                    if !parse {
                        continue;
                    }
                    let mut test = line.split_whitespace().peekable();
                    if test.peek().is_none() {
                        continue;
                    }
                    let input = match test.next() {
                        Some("null") => "",
                        Some(res) => res,
                        None => {
                            panic!(format!(
                                "line {} of the test file doesn't seem to be valid",
                                i
                            ));
                        }
                    };
                    let (expected_root, expected_suffix) = match test.next() {
                        Some("null") => (None, None),
                        Some(root) => {
                            let suffix = {
                                let parts: Vec<&str> = root.split('.').rev().collect();
                                (&parts[..parts.len() - 1])
                                    .iter()
                                    .rev()
                                    .map(|part| *part)
                                    .collect::<Vec<_>>()
                                    .join(".")
                            };
                            (Some(root.to_string()), Some(suffix.to_string()))
                        }
                        None => {
                            panic!(format!(
                                "line {} of the test file doesn't seem to be valid",
                                i
                            ));
                        }
                    };
                    let (found_root, found_suffix) = match LIST.parse_domain(input) {
                        Ok(domain) => {
                            let found_root = match domain.root() {
                                Some(found) => Some(found.to_string()),
                                None => None,
                            };
                            let found_suffix = match domain.suffix() {
                                Some(found) => Some(found.to_string()),
                                None => None,
                            };
                            (found_root, found_suffix)
                        }
                        Err(_) => (None, None),
                    };
                    if expected_root != found_root
                        || (expected_root.is_some() && expected_suffix != found_suffix)
                    {
                        let msg = format!("\n\nGiven `{}`:\nWe expected root domain to be `{:?}` and suffix be `{:?}`\nBut instead, we have `{:?}` as root domain and `{:?}` as suffix.\nWe are on line {} of `test_psl.txt`.\n\n",
                                          input, expected_root, expected_suffix, found_root, found_suffix, i+1);
                        panic!(msg);
                    }
                }
            }
        }
    }));

    rspec::run(&describe("a domain", (), |ctx| {
        ctx.it("should allow fully qualified domain names", |_| {
            assert!(LIST.parse_domain("example.com.").is_ok());
        });

        ctx.it("should not allow more than 1 trailing dot", |_| {
            assert!(LIST.parse_domain("example.com..").is_err());
            match *LIST.parse_domain("example.com..").unwrap_err().kind() {
                ErrorKind::InvalidDomain(ref domain) => assert_eq!(domain, "example.com.."),
                _ => assert!(false),
            }
        });

        ctx.it(
            "should allow a single label with a single trailing dot",
            |_| {
                assert!(LIST.parse_domain("com.").is_ok());
            },
        );

        ctx.it(
            "should always have a suffix for single-label domains",
            |_| {
                let domains = vec![
                    // real TLDs
                    "com",
                    "saarland",
                    "museum.",
                    // non-existant TLDs
                    "localhost",
                    "madeup",
                    "with-dot.",
                ];
                for domain in domains {
                    let res = LIST.parse_domain(domain).unwrap();
                    assert_eq!(res.suffix(), Some(domain.trim_end_matches('.')));
                    assert!(res.root().is_none());
                }
            },
        );

        ctx.it(
            "should have the same result with or without the trailing dot",
            |_| {
                assert_eq!(
                    LIST.parse_domain("com.").unwrap(),
                    LIST.parse_domain("com").unwrap()
                );
            },
        );

        ctx.it("should not have empty labels", |_| {
            assert!(LIST.parse_domain("exa..mple.com").is_err());
        });

        ctx.it("should not contain spaces", |_| {
            assert!(LIST.parse_domain("exa mple.com").is_err());
        });

        ctx.it("should not start with a dash", |_| {
            assert!(LIST.parse_domain("-example.com").is_err());
        });

        ctx.it("should not end with a dash", |_| {
            assert!(LIST.parse_domain("example-.com").is_err());
        });

        ctx.it("should not contain /", |_| {
            assert!(LIST.parse_domain("exa/mple.com").is_err());
        });

        ctx.it("should not have a label > 63 characters", |_| {
            let mut too_long_domain = String::from("a");
            for _ in 0..64 {
                too_long_domain.push_str("a");
            }
            too_long_domain.push_str(".com");
            assert!(LIST.parse_domain(&too_long_domain).is_err());
        });

        ctx.it("should not be an IPv4 address", |_| {
            assert!(LIST.parse_domain("127.38.53.247").is_err());
        });

        ctx.it("should not be an IPv6 address", |_| {
            assert!(LIST
                .parse_domain("fd79:cdcb:38cc:9dd:f686:e06d:32f3:c123")
                .is_err());
        });

        ctx.it(
            "should allow numbers only labels that are not the tld",
            |_| {
                assert!(LIST.parse_domain("127.com").is_ok());
            },
        );

        ctx.it("should not have more than 127 labels", |_| {
            let mut too_many_labels_domain = String::from("a");
            for _ in 0..126 {
                too_many_labels_domain.push_str(".a");
            }
            too_many_labels_domain.push_str(".com");
            assert!(LIST.parse_domain(&too_many_labels_domain).is_err());
        });

        ctx.it("should not have more than 253 characters", |_| {
            let mut too_many_chars_domain = String::from("aaaaa");
            for _ in 0..50 {
                too_many_chars_domain.push_str(".aaaaaa");
            }
            too_many_chars_domain.push_str(".com");
            assert!(LIST.parse_domain(&too_many_chars_domain).is_err());
        });

        ctx.it("should choose the longest valid suffix", |_| {
            let domain = LIST.parse_domain("foo.builder.nu").unwrap();
            assert_eq!(Some("nu"), domain.suffix());
            assert_eq!(Some("builder.nu"), domain.root());

            let domain = LIST.parse_domain("foo.fbsbx.com").unwrap();
            assert_eq!(Some("com"), domain.suffix());
            assert_eq!(Some("fbsbx.com"), domain.root());
        });

        ctx.it(
            "should not indicate wildcard matched domains as having known suffix",
            |_| {
                let domain = LIST.parse_domain("some.total.nonsensetld").unwrap();
                assert!(!domain.has_known_suffix());
            },
        );
    }));

    rspec::run(&describe("a DNS name", (), |ctx| {
        ctx.it("should allow extended characters", |_| {
            let names = vec![
                "_tcp.example.com.",
                "_telnet._tcp.example.com.",
                "*.example.com.",
                "ex!mple.com.",
            ];
            for name in names {
                println!("{} should be valid", name);
                assert!(LIST.parse_dns_name(name).is_ok());
            }
        });

        ctx.it(
            "should allow extracting the correct domain name where possible",
            |_| {
                let names = vec![
                    ("_tcp.example.com.", "example.com"),
                    ("_telnet._tcp.example.com.", "example.com"),
                    ("*.example.com.", "example.com"),
                ];
                for (name, domain) in names {
                    println!("{}'s root domain should be {}", name, domain);
                    let name = LIST.parse_dns_name(name).unwrap();
                    let root = name.domain().unwrap().root();
                    assert_eq!(root, Some(domain));
                }
            },
        );

        ctx.it("should not extract any domain where not possible", |_| {
            let names = vec!["_tcp.com.", "_telnet._tcp.com.", "*.com.", "ex!mple.com."];
            for name in names {
                println!("{} should not have any root domain", name);
                let name = LIST.parse_dns_name(name).unwrap();
                assert!(name.domain().is_none());
            }
        });

        ctx.it("should not allow more than 1 trailing dot", |_| {
            assert!(LIST.parse_dns_name("example.com..").is_err());
            match *LIST.parse_dns_name("example.com..").unwrap_err().kind() {
                ErrorKind::InvalidDomain(ref domain) => assert_eq!(domain, "example.com.."),
                _ => assert!(false),
            }
        });
    }));

    rspec::run(&describe("a host", (), |ctx| {
        ctx.it("can be an IPv4 address", |_| {
            assert!(LIST.parse_host("127.38.53.247").is_ok());
        });

        ctx.it("can be an IPv6 address", |_| {
            assert!(LIST
                .parse_host("fd79:cdcb:38cc:9dd:f686:e06d:32f3:c123")
                .is_ok());
        });

        ctx.it("can be a domain name", |_| {
            assert!(LIST.parse_host("example.com").is_ok());
        });

        ctx.it("cannot be neither an IP address nor a domain name", |_| {
            assert!(LIST.parse_host("23.56").is_err());
        });

        ctx.it("an IPv4 address should parse into an IP object", |_| {
            assert!(LIST.parse_host("127.38.53.247").unwrap().is_ip());
        });

        ctx.it("an IPv6 address should parse into an IP object", |_| {
            assert!(LIST
                .parse_host("fd79:cdcb:38cc:9dd:f686:e06d:32f3:c123")
                .unwrap()
                .is_ip());
        });

        ctx.it("a domain name should parse into a domain object", |_| {
            assert!(LIST.parse_host("example.com").unwrap().is_domain());
        });

        ctx.it("can be parsed from a URL with a domain as hostname", |_| {
            assert!(LIST
                .parse_url("https://publicsuffix.org/list/")
                .unwrap()
                .is_domain());
        });

        ctx.it(
            "can be parsed from a URL with an IP address as hostname",
            |_| {
                assert!(LIST
                    .parse_url("https://127.38.53.247:8080/list/")
                    .unwrap()
                    .is_ip());
            },
        );

        ctx.it("can be parsed from a URL using `parse_str`", |_| {
            assert!(LIST
                .parse_str("https://127.38.53.247:8080/list/")
                .unwrap()
                .is_ip());
        });

        ctx.it("can be parsed from a non-URL using `parse_str`", |_| {
            assert!(LIST.parse_str("example.com").unwrap().is_domain());
        });
    }));

    rspec::run(&describe("a parsed email", (), |ctx| {
        ctx.it("should allow valid email addresses", |_| {
            let emails = vec![
                "prettyandsimple@example.com",
                "very.common@example.com",
                "disposable.style.email.with+symbol@example.com",
                "other.email-with-dash@example.com",
                "x@example.com",
                "example-indeed@strange-example.com",
                "#!$%&'*+-/=?^_`{}|~@example.org",
                "example@s.solutions",
                "user@[fd79:cdcb:38cc:9dd:f686:e06d:32f3:c123]",
                r#""Abc\@def"@example.com"#,
                r#""Fred Bloggs"@example.com"#,
                r#""Joe\\Blow"@example.com"#,
                r#""Abc@def"@example.com"#,
                r#"customer/department=shipping@example.com"#,
                "$A12345@example.com",
                "!def!xyz%abc@example.com",
                "_somename@example.com",
            ];
            for email in emails {
                println!("{} should be valid", email);
                assert!(LIST.parse_email(email).is_ok());
            }
        });

        ctx.it("should reject invalid email addresses", |_| {
            let emails = vec![
                "Abc.example.com",
                "A@b@c@example.com",
                r#"a"b(c)d,e:f;g<h>i[j\k]l@example.com"#,
                r#""just"not"right@example.com"#,
                r#"this is"not\allowed@example.com"#,
                r#"this\ still\"not\\allowed@example.com"#,
                "1234567890123456789012345678901234567890123456789012345678901234+x@example.com",
                "john..doe@example.com",
                "john.doe@example..com",
                " prettyandsimple@example.com",
                "prettyandsimple@example.com ",
            ];
            for email in emails {
                println!("{} should not be valid", email);
                assert!(LIST.parse_email(email).is_err());
            }
        });

        ctx.it("should allow parsing emails as str", |_| {
            assert!(LIST
                .parse_str("prettyandsimple@example.com")
                .unwrap()
                .is_domain());
        });

        ctx.it("should allow parsing emails as URL", |_| {
            assert!(LIST
                .parse_url("mailto://prettyandsimple@example.com")
                .unwrap()
                .is_domain());
        });

        ctx.it("should allow parsing IDN email addresses", |_| {
            let emails = vec![
                r#"Pelé@example.com"#,
                r#"δοκιμή@παράδειγμα.δοκιμή"#,
                r#"我買@屋企.香港"#,
                r#"甲斐@黒川.日本"#,
                r#"чебурашка@ящик-с-апельсинами.рф"#,
                r#"संपर्क@डाटामेल.भारत"#,
                r#"用户@例子.广告"#,
            ];
            for email in emails {
                println!("{} should be valid", email);
                assert!(LIST.parse_email(email).is_ok());
            }
        });
    }));
}
