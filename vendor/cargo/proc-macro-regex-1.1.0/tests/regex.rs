use proc_macro_regex::regex;

#[test]
fn ipv4_regex() {
    // source https://stackoverflow.com/questions/53497/regular-expression-that-matches-valid-ipv6-addresses
    regex!(ipv4 r"^((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])$");
    assert!(ipv4("127.0.0.1"));
    assert!(!ipv4("127.0.0.256"));
}

#[test]
fn ipv6_regex() {
    // source https://stackoverflow.com/questions/53497/regular-expression-that-matches-valid-ipv6-addresses
    regex!(ipv6 r"^(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))$" 1048576);
    assert!(ipv6("fe80::1ff:fe23:4567:890a"));
    assert!(!ipv6("fe80::1ff::fe23:4567:890a"));
}

#[test]
fn ipv6_pattern_regex() {
    // source https://stackoverflow.com/questions/53497/regular-expression-that-matches-valid-ipv6-addresses
    regex!(ipv6 r"(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))");
    assert!(ipv6("Pattern fe80::1ff:fe23:4567:890a"));
}

#[test]
fn email_regex() {
    regex!(email "^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\\.[a-zA-Z0-9-.]+$" 0);
    assert!(email("example@example.org"));
    assert!(!email("example@example@org"));
}

#[test]
fn url_http() {
    // source https://gist.github.com/jacksonfdam/3000275
    regex!(url_http r"^http(s)?://(([0-9]+\.[0-9]+\.[0-9]+\.[0-9]+)|(([0-9A-Za-z-]+\.)+([a-z,A-Z][0-9A-Za-z_-]*)))(:[1-9][0-9]*)?(/([0-9A-Za-z_./:%+@&=-]+[0-9A-Za-z_ ./?:%+@&=-]*)?)?(#([\t\n\v\f\r ]*))?$");
    assert!(url_http("http://127.0.0.1/page?param=value"));
    assert!(url_http("http://www.example.org/page?param=value"));
    assert!(!url_http("htt://www.example.org/"));
}
