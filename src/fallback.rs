#[cfg(feature = "ssr")]
pub use ssr::*;

#[cfg(feature = "ssr")]
mod ssr {
	use axum::response::Response as AxumResponse;
	use axum::{
		body::{boxed, Body, BoxBody},
		extract::Extension,
		http::{Request, Response, StatusCode, Uri},
		response::IntoResponse,
	};
	use leptos::LeptosOptions;
	use std::convert::Infallible;
	use std::sync::Arc;
	use tower::ServiceExt;
	use tower_http::services::ServeDir;

	/// This is the fallback handler for the server. It will serve static files if they exist, and
	/// otherwise render the app.  Leptos will return 200 even on routes that don't exist for note.
	pub async fn file_and_error_handler(
		uri: Uri,
		Extension(options): Extension<Arc<LeptosOptions>>,
		//Extension(db): Extension<Arc<crate::DBPool>>,
		//req: Request<Body>,
	) -> AxumResponse {
		let options = &*options;
		let root = options.site_root.clone();

		match get_static_file(uri.clone(), &root).await {
			Ok(res) if res.status() == StatusCode::OK => res.into_response(),
			_ => {
				// dbg!((&uri, &root));
				// let handler = leptos_axum::render_app_to_stream_with_context(
				// 	options.clone(),
				// 	move |cx| provide_context(cx, db.clone()),
				// 	move |cx| view! {cx, <App/>},
				// );
				// dbg!(handler(req).await).into_response()
				(StatusCode::NOT_FOUND, "Not found").into_response()
			}
		}
	}

	async fn get_static_file(uri: Uri, root: &str) -> Result<Response<BoxBody>, (StatusCode, String)> {
		let req = Request::builder()
			.uri(uri)
			.body(Body::empty())
			.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
		match ServeDir::new(root).oneshot(req).await {
			Ok(resp) => Ok(resp.map(boxed)),
			Err(e) => {
				#[allow(clippy::no_effect_underscore_binding)]
				let _e: Infallible = e;
				unimplemented!("the error return type is `Infallible`");
				// warn!(?e, "Error serving file");
				// // let status = e.status();
				// let status = StatusCode::INTERNAL_SERVER_ERROR;
				// let msg = format!("Error serving file: {e}");
				// Err((status, msg))
			}
		}
	}
}
