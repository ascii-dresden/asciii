use env_logger::{self, Env};

use actix_web::{middleware, web, App, HttpResponse, HttpServer};

use asciii::server::ProjectLoader;
use icalendar::Calendar;

use std::{
    env,
    sync::{mpsc, Mutex},
    thread,
};

// TODO: replace by actor
lazy_static::lazy_static! {
    pub static ref PROJECTS: Mutex<ProjectLoader> = Mutex::new(ProjectLoader::new());

    pub static ref CHANNEL: mpsc::SyncSender<()> = {
        let (tx, rx) = mpsc::sync_channel::<()>(1);

        thread::spawn(move || {
            log::debug!("background thread");
            let mut count = 0;
            loop {
                rx.recv().unwrap();
                count += 1;
                if count % 6 == 0 {
                    log::debug!("updating projects");
                    PROJECTS.lock().unwrap().update();
                }
                log::debug!("call-count: {}", count);
            }
        });
        tx
    };
}

const LOG_VAR: &str = "ASCIII_LOG";
const BIND_VAR: &str = "ASCIII_BIND";
const BIND_HOST: &str = "127.0.0.1";
const BIND_PORT: &str = "8000";

pub mod api {

    use actix_web::{get, http::header, web, HttpRequest, HttpResponse};

    use asciii::{
        project::export::{Complete, ExportTarget},
        storage::Storable,
    };
    use linked_hash_map::LinkedHashMap;
    use serde::Deserialize;

    use super::*;

    #[derive(Deserialize, Debug)]
    pub struct YearRequest {
        year: i32,
    }

    #[derive(Deserialize, Debug)]
    pub struct NameRequest {
        name: String,
    }

    #[get("/version")]
    pub async fn version() -> HttpResponse {
        let version: &str = asciii::VERSION_JSON.as_ref();
        log::info!("version {}", version);
        HttpResponse::Ok()
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .body(version)
    }

    pub mod calendar {
        use super::*;
        use asciii::project::spec::HasEvents;

        #[get("/calendar")]
        pub async fn calendar() -> HttpResponse {
            log::info!("calendar");
            self::CHANNEL.send(()).unwrap();
            let loader = self::PROJECTS.lock().unwrap();

            let mut tasks = Calendar::new();
            for project in loader.state.working.values() {
                tasks.append(&mut project.to_tasks())
            }

            let mut cal = Calendar::new();
            for project in loader.state.all.iter() {
                cal.append(&mut project.to_ical())
            }
            cal.append(&mut tasks);

            HttpResponse::Ok()
                .insert_header((header::CONTENT_TYPE, "text/calendar"))
                .body(cal.to_string())
        }
    }

    pub mod projects {
        use super::*;

        #[get("/year")]
        pub async fn years(_req: HttpRequest) -> HttpResponse {
            log::info!("years");
            self::CHANNEL.send(()).unwrap();
            let loader = self::PROJECTS.lock().unwrap();

            HttpResponse::Ok().json(&loader.state.years)
        }

        #[get("/year/{year}")]
        pub async fn by_year(param: web::Path<YearRequest>) -> HttpResponse {
            log::info!("by_year");
            self::CHANNEL.send(()).unwrap();
            let loader = self::PROJECTS.lock().unwrap();
            let exported = loader
                .state
                .mapped
                .iter()
                .filter(|&(_, p)| {
                    if let Some(y) = Storable::year(p) {
                        y == param.year
                    } else {
                        false
                    }
                })
                .map(|(ident, _p)| ident.as_str())
                .collect::<Vec<&str>>();

            HttpResponse::Ok().json(exported)
        }

        #[get("/{name}")]
        pub async fn by_name(param: web::Path<NameRequest>) -> HttpResponse {
            log::info!("by_name({:?})", param.name);
            self::CHANNEL.send(()).unwrap();
            let loader = self::PROJECTS.lock().unwrap();
            let exported = loader
                .state
                .mapped
                .iter()
                .filter(|&(ident, _p)| *ident == param.name)
                .map(|(ident, p)| {
                    let exported: Complete = p.export();
                    (ident, exported)
                })
                .collect::<LinkedHashMap<_, _>>();

            HttpResponse::Ok().json(exported)
        }

        #[get("/workingdir")]
        pub async fn working_dir() -> HttpResponse {
            log::info!("projects/workingdir");
            let loader = self::PROJECTS.lock().unwrap();
            let list = loader.state.working.iter().map(|(ident, _)| ident).collect::<Vec<_>>();

            self::CHANNEL.send(()).unwrap();

            HttpResponse::Ok().json(&list)
        }

        pub async fn all_names() -> HttpResponse {
            let loader = self::PROJECTS.lock().unwrap();
            let list = loader.state.mapped.iter().map(|(ident, _)| ident).collect::<Vec<_>>();

            self::CHANNEL.send(()).unwrap();

            HttpResponse::Ok().json(&list)
        }
    }

    pub mod full_projects {
        use super::*;

        #[get("/year/{year}")]
        pub async fn by_year(param: web::Path<YearRequest>) -> HttpResponse {
            let loader = self::PROJECTS.lock().unwrap();
            let exported = loader
                .state
                .mapped
                .iter()
                .filter(|&(_, p)| {
                    if let Some(y) = Storable::year(p) {
                        y == param.year
                    } else {
                        false
                    }
                })
                .map(|(ident, p)| {
                    let exported: Complete = p.export();
                    (ident.clone(), exported)
                })
                .collect::<LinkedHashMap<String, Complete>>();

            self::CHANNEL.send(()).unwrap();

            HttpResponse::Ok().json(exported)
        }

        #[get("/workingdir")]
        pub async fn working_dir() -> HttpResponse {
            log::info!("full_projects/workingdir");
            let loader = self::PROJECTS.lock().unwrap();
            let list = loader
                .state
                .working
                .iter()
                .map(|(ident, p)| {
                    let exported: Complete = p.export();
                    (ident, exported)
                })
                .collect::<LinkedHashMap<_, _>>();

            self::CHANNEL.send(()).unwrap();

            HttpResponse::Ok().json(&list)
        }
    }
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_backtrace::install();

    if env::var(LOG_VAR).is_err() {
        //env::set_var(LOG_VAR, "asciii=debug, asciii_web=debug, actix_web=debug");
        env::set_var(LOG_VAR, "asciii=info, asciii_web=debug");
    }
    env_logger::init_from_env(Env::new().filter(LOG_VAR));
    let bind_to = env::var(BIND_VAR).unwrap_or_else(|_| format!("{BIND_HOST}:{BIND_PORT}"));

    log::info!("running asciii-web");
    log::warn!("do not host this on a public server, there is no security by design");

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .service(
                web::scope("api")
                    .service(web::resource("projects").route(web::get().to(api::projects::all_names)))
                    .service(api::version)
                    .service(
                        web::scope("projects")
                            .service(api::projects::years)
                            .service(api::projects::by_year)
                            .service(api::projects::working_dir)
                            .service(api::projects::by_name),
                    )
                    .service(
                        web::scope("full_projects")
                            .service(api::full_projects::by_year)
                            .service(api::full_projects::working_dir)
                            .service(api::projects::years),
                    )
                    .service(api::calendar::calendar),
            )
            // .service(fs::Files::new("/", "webapp/public").index_file("index.html"))
            .service(web::resource("/").route(
                web::get().to(|| async { HttpResponse::Ok().body(include_str!("../../webapp/public/index.html")) }),
            ))
            .service(web::resource("/bundle.css").route(
                web::get().to(|| async { HttpResponse::Ok().body(include_str!("../../webapp/public/bundle.css")) }),
            ))
            .service(web::resource("/bundle.js").route(web::get().to(|| async {
                HttpResponse::Ok()
                    .content_type("application/javascript")
                    .body(include_str!("../../webapp/public/bundle.js"))
            })))
        // .service(
        //     web::resource("/bundle.js.map").route(
        //         web::get().to(|| HttpResponse::Ok().content_type("application/javascript").body(include_str!("../../webapp/public/bundle.js.map")))
        //         ))
    });

    log::info!("listening on http://{}", bind_to);
    server.bind(bind_to)?.run().await?;
    log::info!("shutting down I guess");

    Ok(())
}
