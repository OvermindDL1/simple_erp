#![warn(clippy::pedantic)]

#[cfg(feature = "ssr")]
mod logging;

#[cfg(feature = "ssr")]
mod ssr {
	use anyhow::Context;
	use clap::Parser;
	use leptos::leptos_config::Env;
	use serde::ser::SerializeStruct;
	use std::path::PathBuf;

	#[derive(Parser, Clone, Debug)]
	#[command(author, version, about)]
	pub struct Args {
		#[arg(long, short, env = "ERP_CONFIG_FILE")]
		pub config: Option<PathBuf>,
	}

	fn serialize_leptos_options<S: serde::Serializer>(
		options: &leptos::LeptosOptions,
		s: S,
	) -> Result<S::Ok, S::Error> {
		let mut st = s.serialize_struct("LeptosOptions", 5)?;
		st.serialize_field("output_name", &options.output_name)?;
		st.serialize_field("site_root", &options.site_root)?;
		st.serialize_field("site_pkg_dir", &options.site_pkg_dir)?;
		st.serialize_field(
			"env",
			match &options.env {
				Env::PROD => "PROD",
				Env::DEV => "DEV",
			},
		)?;
		st.serialize_field("site_addr", &options.site_addr)?;
		st.serialize_field("reload_port", &options.reload_port)?;
		st.end()
	}

	#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
	pub struct Config {
		pub logging: crate::logging::LogArgs,
		#[serde(serialize_with = "serialize_leptos_options")]
		pub leptos_options: leptos::LeptosOptions,
		pub database_url: String,
		pub migrate_db_on_load: bool,
	}

	impl Default for Config {
		fn default() -> Self {
			let leptos_conf = leptos::leptos_config::get_config_from_env().unwrap_or_else(|_| {
				let leptos_config = leptos::LeptosOptions {
					output_name: "erpc".to_string(),
					site_root: "target/site".to_string(),
					site_pkg_dir: "pkg".to_string(),
					env: Env::PROD,
					site_addr: std::net::SocketAddr::from(([127, 0, 0, 1], 3000)),
					reload_port: 3001,
				};
				leptos::leptos_config::ConfFile {
					leptos_options: leptos_config,
				}
			});
			Config {
				logging: crate::logging::LogArgs::default(),
				leptos_options: leptos_conf.leptos_options,
				database_url: "sqlite:erp.db".into(),
				migrate_db_on_load: true,
			}
		}
	}

	impl Config {
		fn get_config_path(args: &Args) -> Option<PathBuf> {
			if let Some(path) = &args.config {
				return Some(path.clone());
			}

			let path = PathBuf::from("./erp.toml");
			if path.exists() {
				return Some(path);
			}

			let mut path = dirs::config_dir().unwrap_or_else(|| ".".into());
			path.push("erp.toml");
			if path.exists() {
				return Some(path);
			}

			let path = PathBuf::from("/etc/erp.toml");
			if path.exists() {
				return Some(path);
			}

			None
		}

		pub fn load() -> Result<Self, anyhow::Error> {
			let args = Args::parse();
			if let Some(config_path) = Self::get_config_path(&args) {
				let config_file = std::fs::read_to_string(&config_path)?;
				let config: Self = toml::from_str(&config_file).with_context(|| format!("in file {config_path:?}"))?;
				Ok(config)
			} else {
				std::fs::write("./erp.toml", toml::to_string_pretty(&Self::default())?)?;
				let config_path = dirs::config_dir()
					.unwrap_or_else(|| ".".into())
					.to_string_lossy()
					.into_owned();
				anyhow::bail!("No config file found, created default one at ./erp.toml, please edit it (and optionally move it to either a path given by the `ERP_CONFIG_FILE` environment variable or to {config_path}/erp.toml or to /etc/erp.toml) and restart the server");
			}
		}
	}
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
	use axum::body::Body;
	use axum::extract::{Path, RawQuery};
	use axum::headers::HeaderMap;
	use axum::http::Request;
	use axum::{extract::Extension, routing::post, Router};
	use leptos::provide_context;
	use leptos::view;
	use leptos_axum::{generate_route_list, LeptosRoutes};
	use simple_erp::app::register_server_fns;
	use simple_erp::app::App;
	use simple_erp::DBPool;
	use std::sync::Arc;
	use std::time::Duration;
	use tower_http::services::ServeDir;
	use tracing::info;

	let config = ssr::Config::load()?;
	logging::init_logger(&config.logging)?;
	info!("Launching ERP server");

	let leptos_options = config.leptos_options;
	let addr = leptos_options.site_addr;
	let routes = generate_route_list(|cx| view! { cx, <App/> }).await;

	let db = Arc::new(
		sqlx::sqlite::SqlitePoolOptions::new()
			.max_connections(5)
			.min_connections(2)
			.acquire_timeout(Duration::from_secs(8))
			.idle_timeout(Duration::from_secs(8))
			.max_lifetime(Duration::from_secs(8))
			.connect(&config.database_url)
			.await?,
	);
	let db_routes: Arc<DBPool> = db.clone();
	let db_server_fns: Arc<DBPool> = db.clone();

	if config.migrate_db_on_load {
		sqlx::migrate!("./migrations").run(&*db).await?;
	}

	register_server_fns()?;

	let app = Router::new()
		.route(
			"/api/*fn_name",
			post(
				|Path(fn_name): Path<String>, headers: HeaderMap, RawQuery(query): RawQuery, req: Request<Body>| {
					leptos_axum::handle_server_fns_with_context(
						Path(fn_name),
						headers,
						RawQuery(query),
						move |cx| {
							dbg!("blorp");
							provide_context(cx, db_server_fns.clone());
						},
						req,
					)
				},
			),
		)
		.leptos_routes_with_context(
			leptos_options.clone(),
			routes,
			move |cx| provide_context(cx, db_routes.clone()),
			|cx| {
				dbg!("blah");
				view! { cx, <App/> }
			},
		)
		.fallback_service(ServeDir::new(&leptos_options.site_root))
		// .fallback(simple_erp::fallback::file_and_error_handler)
		.layer(Extension(Arc::new(leptos_options)))
		.layer(Extension(db));

	info!("listening on http://{}", &addr);
	axum::Server::bind(&addr).serve(app.into_make_service()).await?;
	Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
	// No client-side main function unless we want this to work with, e.g., Trunk for pure
	// client-side testing, see lib.rs for hydration function instead
}
