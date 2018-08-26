#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate asciii;
extern crate base64;
#[cfg(feature = "webapp")]
#[macro_use]
extern crate include_dir;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate linked_hash_map;
#[macro_use]
extern crate log;
extern crate openssl_probe;
extern crate rocket;
extern crate rocket_contrib;
extern crate rocket_cors;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use rocket::http::Method;
use rocket::response::NamedFile;
use rocket_cors::{AllowedOrigins, AllowedHeaders};
use asciii::actions;
use asciii::server::ProjectLoader;
use asciii::storage::StorageDir;

use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread;

#[cfg(feature = "webapp")]
static WEBAPP_DIR: include_dir::Dir = include_dir!("./webapp/public");

#[derive(FromForm, Debug)]
struct Dir {
    year: Option<i32>,
    all: Option<bool>,
}

impl Dir {
    fn into_storage_dir(self) -> Result<StorageDir, String> {
        let dir = match self {
            Dir{all: Some(true), year: None} => StorageDir::All,
            Dir{all: Some(true), year: Some(_)} => return Err("Ambiguous".into()),
            Dir{all: None, year: Some(year)} => StorageDir::Archive(year),
            Dir{all: None, year: None} => StorageDir::Working,
            _ => StorageDir::Working,
        };
        Ok(dir)
    }
}

lazy_static! {
    pub static ref PROJECTS: Mutex<ProjectLoader> = Mutex::new(ProjectLoader::new());

    pub static ref CHANNEL: SyncSender<()> = {
        let (tx, rx) = sync_channel::<()>(1);

        thread::spawn(move || {
            debug!("background thread");
            let mut count = 0;
            loop {
                rx.recv().unwrap();
                count += 1;
                if count % 6 == 0 {
                    debug!("updating projects");
                    PROJECTS.lock().unwrap().update();
                }
                debug!("callcount: {}", count);
            }
        });
        tx
    };
}

#[cfg(not(feature = "webapp"))]
#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open("webapp/public/index.html")
}

#[cfg(not(feature = "webapp"))]
#[get("/<file..>", rank=5)]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("webapp/public/").join(file)).ok()
}

#[cfg(feature = "webapp")]
#[get("/")]
/// Index.html directly embedded in the binary
fn index() -> Option<content::Html<&'static str>> {
    WEBAPP_DIR.get_file("index.html").and_then(|file| file.contents_utf8())
    .map(content::Html)
}

#[cfg(feature= "webapp")]
#[get("/<file_path..>", rank=5)]
fn files(file_path: PathBuf) -> Option<content::Content<&'static str>> {
    use rocket::http::ContentType;
    use std::ffi::OsStr;

    WEBAPP_DIR.get_file(&file_path)
    .and_then(|file| file.contents_utf8())
    .map(|file| match file_path.extension()  {
        Some(ext) if ext == OsStr::new("css") => content::Content(ContentType::new("text", "css"), file),
        _ => content::Content(ContentType::new("text", "plain"), file)
    })
}

mod calendar {
    use super::Dir;

    use rocket::response::content::{self, Content};
    use rocket::http::ContentType;

    use asciii::actions;

    #[get("/", rank=2)]
    fn cal() -> Result<content::Content<String>, String> {
        cal_params(Dir{year:None,all:None})
    }

    #[get("/", rank=2)]
    fn cal_plain() -> Result<content::Plain<String>, String> {
        cal_plain_params(Dir{year:None,all:None})
    }

    #[get("/?<dir>", rank=1)]
    fn cal_params(dir: Dir) -> Result<content::Content<String>, String> {
        let storage_dir = dir.into_storage_dir()?;

        actions::calendar(storage_dir)
            .map(|s| Content(ContentType::new("text", "calendar"),s) )
            .map_err(|_|String::from("error"))
    }

    #[get("/?<dir>", rank=1)]
    fn cal_plain_params(dir:Dir) -> Result<content::Plain<String>, String> {
        let storage_dir = dir.into_storage_dir()?;
        actions::calendar(storage_dir)
            .map(|s| content::Plain(s) )
            .map_err(|_|String::from("error"))

    }
}

mod projects {
    use linked_hash_map::LinkedHashMap;
    use asciii::project::export::Complete;
    use asciii::project::export::ExportTarget;
    use asciii::storage::{Storable, Year};
    use serde_json;
    use rocket::response::content;

    #[get("/projects/year")]
    fn years() -> content::Json<String> {
        ::CHANNEL.send(()).unwrap();
        let loader = ::PROJECTS.lock().unwrap();

        content::Json(serde_json::to_string(&loader.state.years).unwrap())
    }

    #[get("/projects/workingdir")]
    fn working_dir() -> content::Json<String> {
        let loader = ::PROJECTS.lock().unwrap();
        let list = loader.state.working.iter()
                         .map(|(ident, p)| {
                             let exported: Complete = p.export();
                             (ident, exported)
                         })
                         .collect::<LinkedHashMap<_,_>>();

        ::CHANNEL.send(()).unwrap();

        content::Json(serde_json::to_string(&list).unwrap())
    }

    #[get("/full_projects/year/<year>")]
    fn full_by_year(year: Year) -> content::Json<String> {
        let loader = ::PROJECTS.lock().unwrap();
        let exported = loader.state.mapped.iter()
            .filter(|&(_, p)| if let Some(y) = Storable::year(p) {y == year } else {false})
            .map(|(ident, p)| {
                let exported: Complete = p.export();
                (ident.clone(), exported)
            })
            .collect::<LinkedHashMap<String, Complete>>();

        ::CHANNEL.send(()).unwrap();

        content::Json(serde_json::to_string(&exported).unwrap())
    }

    #[get("/projects/year/<year>")]
    fn by_year(year: Year) -> content::Json<String> {
        let loader = ::PROJECTS.lock().unwrap();
        let exported = loader.state.mapped.iter()
            .filter(|&(_, p)| if let Some(y) = Storable::year(p) {y == year } else {false})
            .map(|(ident, _)| ident.as_str())
            .collect::<Vec<&str>>();

        ::CHANNEL.send(()).unwrap();

        content::Json(serde_json::to_string(&exported).unwrap())
    }

    #[get("/full_projects")]
    fn all_full(_api_key: ::ApiKey) -> content::Json<String> {
        let loader = ::PROJECTS.lock().unwrap();
        let list = loader.state.mapped.iter()
                         .map(|(ident, p)| {
                             let exported: Complete = p.export();
                             (ident, exported)
                         })
                         .collect::<LinkedHashMap<_,_>>();

        ::CHANNEL.send(()).unwrap();

        content::Json(serde_json::to_string(&list).unwrap())
    }

    #[get("/projects")]
    fn all_names() -> content::Json<String> {
        let loader = ::PROJECTS.lock().unwrap();
        let list = loader.state.mapped.iter()
                         .map(|(ident, _)| ident)
                         .collect::<Vec<_>>();

        ::CHANNEL.send(()).unwrap();

        content::Json(serde_json::to_string(&list).unwrap())
    }

    #[get("/projects/<name>")]
    fn by_name(name: String) -> Option<content::Json<String>> {
        let loader = ::PROJECTS.lock().unwrap();
        let list = loader.state.mapped.iter()
                         .map(|(ident, p)| {
                             let exported: Complete = p.export();
                             (ident, exported)
                         })
                         .collect::<LinkedHashMap<_,_>>();

        ::CHANNEL.send(()).unwrap();

         list.get(&name)
             .map(|p| content::Json(serde_json::to_string( p).unwrap()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ApiKey {
    Key(String),
    UsernamePassword(String, String),
}

use rocket::Outcome;
use rocket::http::Status;
use rocket::request::{self, Request, FromRequest};

impl<'a, 'r> FromRequest<'a, 'r> for ApiKey {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<ApiKey, ()> {
        let auth = request
            .headers()
            .get("Authorization")
            .nth(0)
            .map(|s| s.split_whitespace().nth(1).unwrap())
            .and_then(|s| base64::decode(s.as_bytes()).ok())
            .and_then(|v| String::from_utf8(v).ok())
            .map(|s| s.split(':').map(ToOwned::to_owned).collect::<Vec<_>>());

        if let Some(auth) = auth {
            let authorization = match (auth.get(0), auth.get(1)) {
                (Some(user), Some(pass)) => ApiKey::UsernamePassword(user.to_owned(), pass.to_owned()),
                _ => return Outcome::Failure((Status::BadRequest, ()))
            };

            if validate_authorization(&authorization) {
                Outcome::Success(authorization)
            } else {
                Outcome::Failure((Status::Unauthorized, ()))
            }
        } else {
            error!("{:#?}", request);
            Outcome::Failure((Status::BadRequest, ()))
        }
    }
}

fn validate_authorization(given_key: &ApiKey) -> bool {
    // TODO: load keys at const intervals
    let users = match actions::get_api_keys() {
        Ok(keys) => keys.users,
        Err(e) => {error!("{}", e); return false},
    };
    match *given_key {
        ApiKey::Key(_) => false,
        ApiKey::UsernamePassword(ref user, ref password) => {
            users.iter().any(|(u, p)| u == user && p == password)
        }
    }
}

use rocket::response::content;
#[get("/authorization")]
fn authorization(api_key: ApiKey) -> content::Json<String> {
    content::Json(serde_json::to_string(&api_key).unwrap())
}

#[get("/version")]
fn version() -> content::Json<&'static str> {
    let version: &str = asciii::VERSION_JSON.as_ref();
    content::Json(version)
}

fn main() {
    openssl_probe::init_ssl_cert_env_vars();

    let server = rocket::ignite()
        .mount("/", routes![index, files])
        .mount("/cal/plain", routes![calendar::cal_plain, calendar::cal_plain_params])
        .mount("/cal", routes![calendar::cal, calendar::cal_params])
        .mount("/api", routes![projects::years,
                               projects::by_year,
                               projects::full_by_year,
                               projects::working_dir,
                               projects::all_names,
                               projects::all_full,
                               projects::by_name,
                               authorization,
                               version
        ]);

    if let Ok(env_cors) = std::env::var("CORS_ALLOWED_ORIGINS") {

        debug!("Adding CORS Data {}",env_cors);
        let env_allowed_origins = &[env_cors.as_str()];
        let (allowed_origins, failed_origins) = AllowedOrigins::some(env_allowed_origins);
        assert!(failed_origins.is_empty());
        let options = rocket_cors::Cors {
            allowed_origins: allowed_origins,
            allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
            allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
            allow_credentials: true,
            ..Default::default()
        };

        server.attach(options)
    } else {
        server
    }.launch();

}
