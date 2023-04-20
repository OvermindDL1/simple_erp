#![warn(clippy::pedantic)]

pub mod app;
pub mod fallback;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
	use app::*;
	use leptos::*;
	use tracing_subscriber::fmt;
	use tracing_subscriber_wasm::MakeConsoleWriter;

	// if cfg!(debug_assertions) {
	// 	_ = console_log::init_with_level(log::Level::Debug);
	// } else {
	// 	_ = console_log::init_with_level(log::Level::Info);
	// }
	// console_error_panic_hook::set_once();

	fmt()
		.with_writer(
			// To avoide trace events in the browser from showing their
			// JS backtrace, which is very annoying, in my opinion
			MakeConsoleWriter::default().map_trace_level_to(tracing::Level::DEBUG),
		)
		// For some reason, if we don't do this in the browser, we get
		// a runtime error.
		.without_time()
		.init();

	leptos::mount_to_body(move |cx| view! {cx, <App/>});
}

#[cfg(feature = "ssr")]
pub use ssr::*;
#[cfg(feature = "ssr")]
mod ssr {
	pub type DBPool = sqlx::SqlitePool;
	pub type ArcDBPool = std::sync::Arc<DBPool>;
}
