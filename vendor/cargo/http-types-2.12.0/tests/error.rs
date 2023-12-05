use http_types::{bail, ensure, ensure_eq, Error, StatusCode};
use std::io;

#[test]
fn can_be_boxed() {
    fn can_be_boxed() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let err = io::Error::new(io::ErrorKind::Other, "Oh no");
        Err(Error::new(StatusCode::NotFound, err).into())
    }
    assert!(can_be_boxed().is_err());
}

#[test]
fn internal_server_error_by_default() {
    fn run() -> http_types::Result<()> {
        Err(io::Error::new(io::ErrorKind::Other, "Oh no").into())
    }
    let err = run().unwrap_err();
    assert_eq!(err.status(), 500);
}

#[test]
fn ensure() {
    fn inner() -> http_types::Result<()> {
        ensure!(true, "Oh yes");
        bail!("Oh no!");
    }
    let res = inner();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.status(), StatusCode::InternalServerError);
}

#[test]
fn ensure_eq() {
    fn inner() -> http_types::Result<()> {
        ensure_eq!(1, 1, "Oh yes");
        bail!("Oh no!");
    }
    let res = inner();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.status(), StatusCode::InternalServerError);
}

#[test]
fn result_ext() {
    use http_types::Status;
    fn run() -> http_types::Result<()> {
        let err = io::Error::new(io::ErrorKind::Other, "Oh no");
        Err(err).status(StatusCode::NotFound)?;
        Ok(())
    }
    let res = run();
    assert!(res.is_err());

    let err = res.unwrap_err();
    assert_eq!(err.status(), StatusCode::NotFound);
}

#[test]
fn option_ext() {
    use http_types::Status;
    fn run() -> http_types::Result<()> {
        None.status(StatusCode::NotFound)
    }
    let res = run();
    assert!(res.is_err());

    let err = res.unwrap_err();
    assert_eq!(err.status(), StatusCode::NotFound);
}

#[test]
fn anyhow_error_into_http_types_error() {
    let anyhow_error =
        anyhow::Error::new(std::io::Error::new(std::io::ErrorKind::Other, "irrelevant"));
    let http_types_error: Error = anyhow_error.into();
    assert_eq!(http_types_error.status(), StatusCode::InternalServerError);

    let anyhow_error =
        anyhow::Error::new(std::io::Error::new(std::io::ErrorKind::Other, "irrelevant"));
    let http_types_error: Error = Error::new(StatusCode::ImATeapot, anyhow_error);
    assert_eq!(http_types_error.status(), StatusCode::ImATeapot);
}

#[test]
fn normal_error_into_http_types_error() {
    let http_types_error: Error =
        std::io::Error::new(std::io::ErrorKind::Other, "irrelevant").into();
    assert_eq!(http_types_error.status(), StatusCode::InternalServerError);

    let http_types_error = Error::new(
        StatusCode::ImATeapot,
        std::io::Error::new(std::io::ErrorKind::Other, "irrelevant"),
    );
    assert_eq!(http_types_error.status(), StatusCode::ImATeapot);
}

#[test]
fn u16_into_status_code_in_http_types_error() {
    let http_types_error = Error::new(404, io::Error::new(io::ErrorKind::Other, "Not Found"));
    let http_types_error2 = Error::new(
        StatusCode::NotFound,
        io::Error::new(io::ErrorKind::Other, "Not Found"),
    );
    assert_eq!(http_types_error.status(), http_types_error2.status());

    let http_types_error = Error::from_str(404, "Not Found");
    assert_eq!(http_types_error.status(), StatusCode::NotFound);
}

#[test]
#[should_panic]
fn fail_test_u16_into_status_code_in_http_types_error_new() {
    let _http_types_error = Error::new(
        1000,
        io::Error::new(io::ErrorKind::Other, "Incorrect status code"),
    );
}

#[test]
#[should_panic]
fn fail_test_u16_into_status_code_in_http_types_error_from_str() {
    let _http_types_error = Error::from_str(1000, "Incorrect status code");
}
