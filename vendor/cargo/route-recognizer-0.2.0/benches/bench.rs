#![feature(test)]

extern crate route_recognizer;
extern crate test;

use route_recognizer::Router;

#[bench]
fn benchmark(b: &mut test::Bencher) {
    let mut router = Router::new();
    router.add("/posts/:post_id/comments/:id", "comment".to_string());
    router.add("/posts/:post_id/comments", "comments".to_string());
    router.add("/posts/:post_id", "post".to_string());
    router.add("/posts", "posts".to_string());
    router.add("/comments", "comments2".to_string());
    router.add("/comments/:id", "comment2".to_string());

    b.iter(|| router.recognize("/posts/100/comments/200"));
}
