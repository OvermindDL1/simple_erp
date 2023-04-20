#![warn(clippy::pedantic)]

pub mod app;
pub mod fallback;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
	use app::*;
	use leptos::*;

	if cfg!(debug_assertions) {
		_ = console_log::init_with_level(log::Level::Debug);
	} else {
		_ = console_log::init_with_level(log::Level::Info);
	}
	console_error_panic_hook::set_once();

	leptos::mount_to_body(move |cx| view! {cx, <App/>});
}

#[cfg(feature = "ssr")]
pub use ssr::*;
#[cfg(feature = "ssr")]
mod ssr {
	pub type DBPool = sqlx::SqlitePool;
	pub type ArcDBPool = std::sync::Arc<DBPool>;
}
