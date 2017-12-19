#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate asciii;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;
extern crate linked_hash_map;
extern crate itertools;

extern crate rocket;
extern crate rocket_contrib;
extern crate base64;

use rocket::response::NamedFile;
use itertools::Itertools;

use asciii::project::Project;
use asciii::storage::{self, ProjectList, Storage, StorageDir, Storable};
use linked_hash_map::LinkedHashMap;

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread;


pub struct ProjectLoader {
    storage: Storage<Project>,
    years: Vec<i32>,
    projects_all: ProjectList<Project>,
    projects_map: LinkedHashMap<String, Project>,
}

impl<'a> ProjectLoader {
    pub fn new() -> Self {

        let storage = storage::setup().unwrap();
        let projects_all = storage.open_projects(StorageDir::All).unwrap();
        let projects_map = storage.open_projects(StorageDir::All)
            .unwrap()
            .into_iter()
            .map(|p| (format!("{}-{}",
                              Storable::year(&p).unwrap(),
                              Storable::ident(&p)),
                              p))
            .collect();
        let years = projects_all.iter()
                                    .filter_map(|p: &Project| p.year())
                                    .unique()
                                    .collect::<Vec<_>>();

        Self {
            storage,
            years,
            projects_all,
            projects_map,
        }
    }

    pub fn update(&mut self) {
        debug!("updating projects");
        self.projects_all = self.storage.open_projects(StorageDir::All).unwrap();
    }
}

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
            println!("background thread");
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

#[get("/<file..>", rank=5)]
fn static_files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
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
        content::Json(serde_json::to_string(&loader.years).unwrap())
    }

    #[get("/full_projects/year/<year>")]
    fn full_by_year(year: Year) -> content::Json<String> {
        ::CHANNEL.send(()).unwrap();
        let loader = ::PROJECTS.lock().unwrap();
        let exported = loader.projects_map.iter()
            .filter(|&(_, p)| if let Some(y) = Storable::year(p) {y == year } else {false})
            .map(|(ident, p)| {
                let exported: Complete = p.export();
                (ident.clone(), exported)
            })
            .collect::<LinkedHashMap<String, Complete>>();

        content::Json(serde_json::to_string(&exported).unwrap())
    }

    #[get("/projects/year/<year>")]
    fn by_year(year: Year) -> content::Json<String> {
        ::CHANNEL.send(()).unwrap();
        let loader = ::PROJECTS.lock().unwrap();
        let exported = loader.projects_map.iter()
            .filter(|&(_, p)| if let Some(y) = Storable::year(p) {y == year } else {false})
            .map(|(ident, _)| ident.as_str())
            .collect::<Vec<&str>>();

        content::Json(serde_json::to_string(&exported).unwrap())
    }

    #[get("/full_projects")]
    fn all_full(api_key: ::ApiKey) -> content::Json<String> {
        let loader = ::PROJECTS.lock().unwrap();
        let list = loader.projects_map.iter()
                         .map(|(ident, p)| {
                             let exported: Complete = p.export();
                             (ident, exported)
                         })
                         .collect::<LinkedHashMap<_,_>>();

        content::Json(serde_json::to_string(&list).unwrap())
    }

    #[get("/projects")]
    fn all_names() -> content::Json<String> {
        let loader = ::PROJECTS.lock().unwrap();
        let list = loader.projects_map.iter()
                         .map(|(ident, _)| ident)
                         .collect::<Vec<_>>();

        content::Json(serde_json::to_string(&list).unwrap())
    }

    #[get("/projects/<name>")]
    fn by_name(name: String) -> Option<content::Json<String>> {
        let loader = ::PROJECTS.lock().unwrap();
        let list = loader.projects_map.iter()
                         .map(|(ident, p)| {
                             let exported: Complete = p.export();
                             (ident, exported)
                         })
                         .collect::<LinkedHashMap<_,_>>();

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
            match (auth.get(0), auth.get(1)) {
                (Some(user), Some(pass)) => Outcome::Success(ApiKey::UsernamePassword(user.to_owned(), pass.to_owned())),
                _ => Outcome::Failure((Status::BadRequest, ()))
            }
        } else {
            error!("{:#?}", request);
            Outcome::Failure((Status::BadRequest, ()))
        }
    }
}

use rocket::response::content::{self, Content};
#[get("/authorization")]
fn authorization(api_key: ApiKey) -> content::Json<String> {

    content::Json(serde_json::to_string(&api_key).unwrap())
}

fn main() {
    rocket::ignite()
        .mount("/", routes![static_files])
        .mount("/cal/plain", routes![calendar::cal_plain, calendar::cal_plain_params])
        .mount("/cal", routes![calendar::cal, calendar::cal_params])
        .mount("/api", routes![projects::years,
                               projects::by_year,
                               projects::full_by_year,
                               projects::all_names,
                               projects::all_full,
                               projects::by_name,
                               authorization,
        ])
        .launch();
}
