extern crate time;

extern crate http;
extern crate iron;
extern crate "iron-test" as iron_test;
extern crate "static" as static_file;


use time::Timespec;

use http::method::Get;
use iron::{Handler, Url};
use iron_test::{mock, ProjectBuilder};
use static_file::StaticWithCache;


#[test]
fn it_should_return_cache_headers() {
    let p = ProjectBuilder::new("example").file("file1.html", "this is file1");
    p.build();

    let st = StaticWithCache::new(p.root());

    let mut req = mock::request::at(Get, Url::parse("http://localhost:3000/file1.html").unwrap());
    match st.call(&mut req) {
        Ok(res) => {
            assert!(res.headers.cache_control.is_some());
            assert!(res.headers.last_modified.is_some());
        },
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn it_should_return_the_file_if_client_sends_no_modified_time() {
    let p = ProjectBuilder::new("example").file("file1.html", "this is file1");
    p.build();

    let st = StaticWithCache::new(p.root());

    let mut req = mock::request::at(Get, Url::parse("http://localhost:3000/file1.html").unwrap());

    match st.call(&mut req) {
        Ok(res) => assert_eq!(res.status.unwrap().code(), 200),
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn it_should_return_the_file_if_client_has_old_version() {
    let p = ProjectBuilder::new("example").file("file1.html", "this is file1");
    p.build();

    let st = StaticWithCache::new(p.root());

    let mut req = mock::request::at(Get, Url::parse("http://localhost:3000/file1.html").unwrap());

    let now = time::get_time();
    let one_hour_ago = Timespec::new(now.sec - 3600, now.nsec);
    req.headers.if_modified_since = Some(time::at(one_hour_ago));

    match st.call(&mut req) {
        Ok(res) => assert_eq!(res.status.unwrap().code(), 200),
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn it_should_return_304_if_client_has_file_cached() {
    let p = ProjectBuilder::new("example").file("file1.html", "this is file1");
    p.build();

    let st = StaticWithCache::new(p.root());

    let mut req = mock::request::at(Get, Url::parse("http://localhost:3000/file1.html").unwrap());
    req.headers.if_modified_since = Some(time::now_utc());

    match st.call(&mut req) {
        Ok(res) => assert_eq!(res.status.unwrap().code(), 304),
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn it_should_cache_index_html_for_directory_path() {
    let p = ProjectBuilder::new("example").file("dir/index.html", "this is index");
    p.build();

    let st = StaticWithCache::new(p.root());

    let mut req = mock::request::at(Get, Url::parse("http://localhost:3000/dir/").unwrap());
    req.headers.if_modified_since = Some(time::now_utc());

    match st.call(&mut req) {
        Ok(res) => assert_eq!(res.status.unwrap().code(), 304),
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn it_should_defer_to_static_handler_if_directory_misses_trailing_slash() {
    let p = ProjectBuilder::new("example").file("dir/index.html", "this is index");
    p.build();

    let st = StaticWithCache::new(p.root());

    let mut req = mock::request::at(Get, Url::parse("http://localhost:3000/dir").unwrap());
    req.headers.if_modified_since = Some(time::now_utc());

    match st.call(&mut req) {
        Ok(res) => {
            assert_eq!(res.status.unwrap().code(), 301);
            assert!(res.headers.last_modified.is_none());
        },
        Err(e) => panic!("{}", e)
    }
}
