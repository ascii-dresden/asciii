use rocket::response::NamedFile;
use std::path::{Path, PathBuf};

#[get("/", rank = 4)]
pub fn static_index() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/index.html")).ok()
}

#[get("/<file..>", rank = 5)]
pub fn static_files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

pub mod calendar {
    use super::super::Dir;
    use Authorization;

    use rocket::http::ContentType;
    use rocket::response::content::{self, Content};

    use asciii::actions;

    use std::convert::TryInto;

    #[get("/", rank = 2)]
    fn cal(authorization: Authorization) -> Result<content::Content<String>, String> {
        cal_params(authorization, Dir { year: None, all:  None })
    }

    #[get("/", rank = 2)]
    fn cal_plain(authorization: Authorization) -> Result<content::Plain<String>, String> {
        cal_plain_params(authorization, Dir { year: None, all:  None })
    }

    #[get("/?<dir>", rank = 1)]
    fn cal_params(_authorization: Authorization, dir: Dir) -> Result<content::Content<String>, String> {
        let storage_dir = dir.try_into()?;

        actions::calendar(storage_dir)
            .map(|s| Content(ContentType::new("text", "calendar"), s))
            .map_err(|_| String::from("error"))
    }

    #[get("/?<dir>", rank = 1)]
    fn cal_plain_params(_authorization: Authorization, dir: Dir) -> Result<content::Plain<String>, String> {
        let storage_dir = dir.try_into()?;
        actions::calendar(storage_dir)
            .map(|s| content::Plain(s))
            .map_err(|_| String::from("error"))
    }
}

pub mod projects {
    use asciii::project::export::Complete;
    use asciii::project::export::ExportTarget;
    use asciii::storage::{Storable, Year};
    use linked_hash_map::LinkedHashMap;
    use rocket::response::content;
    use serde_json;
    use Authorization;

    #[get("/projects/year")]
    fn years(_authorization: Authorization) -> content::Json<String> {
        ::CHANNEL.send(()).unwrap();
        let loader = ::PROJECTS.lock().unwrap();
        content::Json(serde_json::to_string(&loader.years).unwrap())
    }

    #[get("/full_projects/year/<year>")]
    fn full_by_year(_authorization: Authorization, year: Year) -> content::Json<String> {
        ::CHANNEL.send(()).unwrap();
        let loader = ::PROJECTS.lock().unwrap();
        let exported = loader
            .projects_map
            .iter()
            .filter(|&(_, p)| {
                if let Some(y) = Storable::year(p) {
                    y == year
                } else {
                    false
                }
            })
            .map(|(ident, p)| {
                let exported: Complete = p.export();
                (ident.clone(), exported)
            })
            .collect::<LinkedHashMap<String, Complete>>();

        content::Json(serde_json::to_string(&exported).unwrap())
    }

    #[get("/projects/year/<year>")]
    fn by_year(_authorization: Authorization, year: Year) -> content::Json<String> {
        ::CHANNEL.send(()).unwrap();
        let loader = ::PROJECTS.lock().unwrap();
        let exported = loader
            .projects_map
            .iter()
            .filter(|&(_, p)| {
                if let Some(y) = Storable::year(p) {
                    y == year
                } else {
                    false
                }
            })
            .map(|(ident, _)| ident.as_str())
            .collect::<Vec<&str>>();

        content::Json(serde_json::to_string(&exported).unwrap())
    }

    #[get("/full_projects")]
    fn all_full(_authorization: Authorization) -> content::Json<String> {
        let loader = ::PROJECTS.lock().unwrap();
        let list = loader
            .projects_map
            .iter()
            .map(|(ident, p)| {
                let exported: Complete = p.export();
                (ident, exported)
            })
            .collect::<LinkedHashMap<_, _>>();

        content::Json(serde_json::to_string(&list).unwrap())
    }

    #[get("/projects")]
    fn all_names(_authorization: Authorization) -> content::Json<String> {
        let loader = ::PROJECTS.lock().unwrap();
        let list = loader.projects_map.iter().map(|(ident, _)| ident).collect::<Vec<_>>();

        content::Json(serde_json::to_string(&list).unwrap())
    }

    #[get("/projects/<name>")]
    fn by_name(_authorization: Authorization, name: String) -> Option<content::Json<String>> {
        let loader = ::PROJECTS.lock().unwrap();
        let list = loader
            .projects_map
            .iter()
            .map(|(ident, p)| {
                let exported: Complete = p.export();
                (ident, exported)
            })
            .collect::<LinkedHashMap<_, _>>();

        list.get(&name)
            .map(|p| content::Json(serde_json::to_string(p).unwrap()))
    }
}

