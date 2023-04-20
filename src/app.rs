// This is actually used, just within the `server` procmacro that isn't
// referencing types correctly..
#[allow(unused_imports)]
use leptos::server_fn::ServerFn;
// Because this is what `leptos::tracing` does...
#[cfg(debug_assertions)]
use leptos::tracing;
#[cfg(feature = "ssr")]
use leptos::ServerFn as _;
use leptos::{component, create_resource, server, server_fn, view, IntoView, Scope, ServerFnError, Suspense};
use leptos_meta::{provide_meta_context, Stylesheet, Title};
use leptos_router::{Route, Router, Routes, SsrMode};

#[allow(clippy::module_name_repetitions)]
#[component]
pub fn App(cx: Scope) -> impl IntoView {
	provide_meta_context(cx);

	view! { cx,
		<Stylesheet id="erpc" href="/pkg/erpc.css" />
		<Title text="Simple ERP" />

		<Router>
			<main>
				<Routes>
					<Route path="" ssr=SsrMode::OutOfOrder view=|cx| view! {cx, <HomePage/> } />
				</Routes>
			</main>
		</Router>
	}
}

#[component]
fn HomePage(cx: Scope) -> impl IntoView {
	let users_testing_todo = create_resource(
		cx,
		|| (),
		move |()| async move {
			dbg!("Hmm");
			get_users(cx).await
		},
	);
	dbg!("Vwooom");
	view! { cx,
		<h1>"Home"</h1>
		<Suspense fallback=move || view! {cx, <p>"Loading..."</p> }>
			{move || {
				users_testing_todo.with(cx, move |users| match users {
					Err(e) => vec![view! {cx, <p>"Server Error: "<span>{e.to_string()}</span></p> }.into_any()],
					Ok(users) => {
						if users.is_empty() {
							vec![view! {cx, <p>"No users found"</p> }.into_any()]
						} else {
							users.iter().map(|(username, name)| {
								view! {cx,
									<p>"User: "<span>{username}</span>" "<span>{name}</span></p>
								}.into_any()
							}).collect::<Vec<_>>()
						}
					}
				})
			}}
		</Suspense>
	}
}

/// # Errors
///
/// This will error if it's somehow unable to register the client-callable server functions.
#[cfg(feature = "ssr")]
pub fn register_server_fns() -> Result<(), ServerFnError> {
	GetUsers::register()?;
	Ok(())
}

/// # Errors
///
/// This will error if the database doesn't exist in the current leptos context, it was forgotten
/// somewhere and should be added.
#[cfg(feature = "ssr")]
pub fn db(cx: Scope) -> Result<crate::ArcDBPool, ServerFnError> {
	leptos::use_context::<crate::ArcDBPool>(cx)
		.ok_or_else(|| ServerFnError::ServerError("database does not exist in the context".to_string()))
}

#[server(GetUsers, "/api")]
async fn get_users(cx: Scope) -> Result<Vec<(String, String)>, ServerFnError> {
	tokio::time::sleep(std::time::Duration::from_secs(1)).await;
	let db = db(cx)?;
	let users = sqlx::query!("SELECT username, name FROM users")
		.fetch_all(&*db)
		.await
		.map_err(|e| {
			tracing::debug!("database error: {e:?}");
			ServerFnError::ServerError("database error".to_string())
		})?
		.into_iter()
		.map(|u| (u.username, u.name))
		.collect();
	dbg!(&users);
	Ok(users)
}
