use crate::locator::{Error, Locator};

#[test]
fn locator_as_base() {
    let loc = Locator::try_from("file://a/b").expect("expected a locator");
    let loc = loc.as_base();
    assert_eq!(loc.url().as_str(), "file://a/b/");
}

#[test]
fn locator_join() {
    let loc = Locator::try_from("file://a/b").expect("expected a locator");
    let loc = loc.join("c").unwrap();
    assert_eq!(loc.url().as_str(), "file://a/c");

    let loc = Locator::try_from("file://a/b/").expect("expected a locator");
    let loc = loc.join("c").unwrap();
    assert_eq!(loc.url().as_str(), "file://a/b/c");

    let loc = Locator::try_from("file://a").expect("expected a locator");
    let err = loc.join("").expect_err("expected an error");
    assert!(matches!(err, Error::EmptyPath));
}
