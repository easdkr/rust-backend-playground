use tower_http::catch_panic::CatchPanicLayer;

pub fn layer() -> CatchPanicLayer<tower_http::catch_panic::DefaultResponseForPanic> {
    CatchPanicLayer::new()
}
