//! Manages file structure of templates, working directory and archives.
//!
//! This module takes care of project file management.
//!
//! Your ordinary file structure would look something like this:
//!
//! ```bash
//! # root dir
//! ├── working
//! │   └── Project1
//! │       └── Project1.yml
//! ├── archive
//! │   ├── 2013
//! │   └── 2014
//! │       └── R036_Project3
//! │           ├── Project3.yml
//! │           └── R036 Project3 2014-10-08.tex
//! ...
//! ```
//!

use rayon::prelude::*;

use std::fs;
use std::env::{self, home_dir, current_dir};
use std::path::{Path, PathBuf};
use std::marker::PhantomData;

/// Year = `i32`
pub type Year =  i32;

/// Result returned by Storage
/// TODO move to `error` module
pub type StorageResult<T> = Result<T, StorageError>;

#[cfg(test)] mod tests;
#[cfg(test)] mod realworld;

mod project_list;
pub use self::project_list::{ProjectList, ProjectsByYear, Projects};
pub mod repo;
pub mod error;
pub use self::error::{StorageError, ErrorKind};
pub mod storable;
pub use self::storable::Storable;


// TODO rely more on IoError, it has most of what you need
/// Manages project file storage.
///
/// This includes:
///
/// * keeping current projects in a working directory
/// * listing project folders and files
/// * listing templates
/// * archiving and unarchiving projects
/// * git interaction
pub struct Storage<L:Storable> {
    /// Root of the entire Structure.
    root:  PathBuf,

    /// Place for project directories.
    working:  PathBuf,

    /// Place for archive directories (*e.g. `2015/`*) each containing project directories.
    archive:  PathBuf,

    /// Place for template files.
    templates: PathBuf,

    /// Place for extra files.
    extras: PathBuf,

    project_type: PhantomData<L>,

    repository: Option<Repository>
}

/// Used to identify what directory you are talking about.
#[derive(Debug,Clone,Copy)]
pub enum StorageDir {
    /// Describes exclusively the working directory.
    Working,
    /// Describes exclusively one year's archive.
    Archive(Year),

    /// Describes archive of year and working directory,
    /// if this year is still current.
    Year(Year),

    /// Parent of `Working`, `Archive` and `Templates`.
    Root,

    /// Place to store templates.
    Templates,

    /// Place to store extra.
    Extras,
    /// `Archive` and `Working` directory, not `Templates`.
    All
}

/// A description from which we can open Storables
#[derive(Debug, Clone)]
pub enum StorageSelection {
    DirAndSearch(StorageDir, Vec<String>),
    Dir(StorageDir),
    Paths(Vec<PathBuf>),
    Unintiailzed
}

impl<'a> Into<StorageSelection> for &'a StorageSelection {
    fn into(self) -> StorageSelection {
        self.clone()
    }
}


impl Into<StorageSelection> for StorageDir {
    fn into(self) -> StorageSelection {
        StorageSelection::Dir(self)
    }
}


impl Default for StorageSelection {
    fn default() -> Self {
        StorageSelection::DirAndSearch(StorageDir::Working, Vec::new())
    }
}

#[cfg_attr(feature = "serialization", derive(Serialize))]
#[derive(Debug)]
pub struct Paths {
    pub storage:   PathBuf,
    pub working:   PathBuf,
    pub archive:   PathBuf,
    pub templates: PathBuf
}

/// Basically `ls`, returns a list of paths.
pub fn list_path_content(path:&Path) -> StorageResult<Vec<PathBuf>> {
    if !path.exists() {
        error!("Path does not exist: {}", path.display());
    }

    Ok(fs::read_dir(path)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .collect::<Vec<PathBuf>>())
}

fn replace_home_tilde(p:&Path) -> PathBuf{
    let path = p.to_str().unwrap();
    PathBuf::from( path.replace("~",home_dir().unwrap().to_str().unwrap()))
}

/// Interprets storage path from config.
///
/// Even if it starts with `~` or is a relatetive path.
/// This is by far the most important function of all utility functions.
pub fn get_storage_path() -> PathBuf
{
    let storage_path = PathBuf::from(::CONFIG.var_get_str("path"))
            .join(::CONFIG.var_get_str("dirs/storage"));

    // TODO make replace tilde a Trait function
    let storage_path = replace_home_tilde(&storage_path);

    if !storage_path.is_absolute(){
        current_dir().unwrap().join(storage_path)
    } else {
        storage_path
    }
}


/// Sets up an instance of `Storage`.
pub fn setup<L:Storable>() -> StorageResult<Storage<L>> {
    trace!("storage::setup()");
    let working   = ::CONFIG.get_str_or("dirs/working").ok_or("Faulty config: dirs/working does not contain a value")?;
    let archive   = ::CONFIG.get_str_or("dirs/archive").ok_or("Faulty config: dirs/archive does not contain a value")?;
    let templates = ::CONFIG.get_str_or("dirs/templates").ok_or("Faulty config: dirs/templates does not contain a value")?;
    let storage   = Storage::new(get_storage_path(), working, archive, templates)?;
    storage.health_check()?;
    Ok(storage)
}

/// Sets up an instance of `Storage`, with git turned on.
pub fn setup_with_git<L:Storable>() -> StorageResult<Storage<L>> {
    trace!("storage::setup_with_git()");
    let working   = ::CONFIG.get_str_or("dirs/working").ok_or("Faulty config: dirs/working does not contain a value")?;
    let archive   = ::CONFIG.get_str_or("dirs/archive").ok_or("Faulty config: dirs/archive does not contain a value")?;
    let templates = ::CONFIG.get_str_or("dirs/templates").ok_or("Faulty config: dirs/templates does not contain a value")?;
    let storage   = if env::var("ASCIII_NO_GIT").is_ok() {
        Storage::new(get_storage_path(), working, archive, templates)?
    } else {
        Storage::new_with_git(get_storage_path(), working, archive, templates)?
    };

    storage.health_check()?;
    Ok(storage)
}



use self::repo::Repository;

use std::fmt;
use std::ffi::OsStr;
use std::ops::DerefMut;
use std::collections::HashMap;
use linked_hash_map::LinkedHashMap;

use slug;
fn slugify(string:&str) -> String{ slug::slugify(string) }

impl<L:Storable> Storage<L> {

    /// Inits storage, does not check existence, yet. TODO
    pub fn new<P: AsRef<Path>>(root:P, working:&str, archive:&str, template:&str) -> StorageResult<Self> {
        trace!("initializing storage, root: {}", root.as_ref().display());
        let root = root.as_ref();
        if root.is_absolute(){
            Ok( Storage{
                root:      root.to_path_buf(),
                working:   root.join(working),
                archive:   root.join(archive),
                templates: root.join(template),
                extras:    root.join("extras"),
                project_type: PhantomData,
                repository: None,
            })
        } else {
            bail!(ErrorKind::StoragePathNotAbsolute)
        }
    }

    /// Inits storage with git capabilities.
    pub fn new_with_git<P: AsRef<Path>>(root:P, working:&str, archive:&str, template:&str) -> StorageResult<Self> {
        trace!("initializing storage, with git");
        Ok( Storage{
            repository: Some(Repository::new(root.as_ref())?),
            .. Self::new(root, working, archive, template)?
        })
    }

    /// Checks whether the folder structure is as it's supposed to be.
    pub fn health_check(&self) -> StorageResult<()> {
        let r = self.root_dir();
        let w = self.working_dir();
        let a = self.archive_dir();
        let t = self.templates_dir();

        if r.exists() && w.exists() && a.exists() && t.exists() {
            Ok(())
        } else {
            for f in &[r,w,a,t]{
                if !f.exists() { warn!("{} does not exist", f.display())}
            }
            bail!(ErrorKind::InvalidDirStructure)
        }
    }

    /// Getter for Storage::storage.
    pub fn root_dir(&self) -> &Path {
        self.root.as_ref()
    }

    /// Getter for Storage::working.
    pub fn working_dir(&self) -> &Path {
        self.working.as_ref()
    }

    /// Getter for Storage::archive.
    pub fn archive_dir(&self) -> &Path {
        self.archive.as_ref()
    }

    /// Getter for Storage::templates.
    pub fn templates_dir(&self) -> &Path {
        self.templates.as_ref()
    }

    /// Getter for Storage::extras.
    pub fn extras_dir(&self) -> &Path {
        self.extras.as_ref()
    }

    /// Getter for Storage::templates.
    pub fn repository(&self) -> Option<&Repository> {
        self.repository.as_ref()
    }

    /// Getter for Storage::templates, returns `Result`.
    pub fn get_repository(&self) -> StorageResult<&Repository> {
        self.repository.as_ref().ok_or(ErrorKind::RepoUnintialized.into())
    }

    /// Returns a struct containing all configured paths of this `Storage`.
    pub fn paths(&self) -> Paths {
        Paths {
           storage: self.root_dir().into(),
           working: self.working_dir().into(),
           archive: self.archive_dir().into(),
           templates: self.templates_dir().into(),
        }
    }

    /// Creates the basic dir structure inside the storage directory.
    ///
    ///<pre>
    ///└── root
    ///    ├── archive
    ///    ├── templates
    ///    └── working
    ///</pre>
    /// If the directories already exist as expected, that's fine
    /// TODO ought to fail when storage_dir already contains directories that do not correspond
    /// with the names given in this setup.
    pub fn create_dirs(&self) -> StorageResult<()> {
        trace!("creating storage directories");
        ensure!(self.root_dir().is_absolute(), ErrorKind::StoragePathNotAbsolute);

        if !self.root_dir().exists()  { fs::create_dir(&self.root_dir())?;  }
        if !self.working_dir().exists()  { fs::create_dir(&self.working_dir())?;  }
        if !self.archive_dir().exists()  { fs::create_dir(&self.archive_dir())?;  }
        if !self.templates_dir().exists() { fs::create_dir(&self.templates_dir())?; }

        Ok(())
    }

    /// Creates an archive for a certain year.
    /// This is a subdirectory under the archive directory.
    ///<pre>
    ///└── root
    ///    ├── archive
    ///        ├── 2001
    ///    ...
    ///</pre>
    pub fn create_archive(&self, year:Year) -> StorageResult<PathBuf> {
        trace!("creating archive directory: {}", year);
        assert!(self.archive_dir().exists());
        let archive = &self.archive_dir().join(year.to_string());

        if self.archive_dir().exists() && !archive.exists() {
            fs::create_dir(archive)?;
        }
        Ok(archive.to_owned())
    }

    /// Produces a list of files in the `extras_dir()`
    pub fn list_extra_files(&self) -> StorageResult<Vec<PathBuf>> {
        trace!("listing extra files");
        list_path_content(&self.extras_dir())
    }

    /// Returns the Path to the extra file by the given name, maybe.
    pub fn get_extra_file(&self, name: &str) -> StorageResult<PathBuf> {
        let full_path = self.extras_dir().join(name);
        trace!("opening {:?}", full_path);

        Ok(full_path)
    }

    /// Produces a list of files in the `template_dir()`
    pub fn list_template_files(&self) -> StorageResult<Vec<PathBuf>> {
        // TODO this is the only reference to `CONFIG`, lets get rid of it
        let template_file_extension = ::CONFIG.get_str("extensions/project_template");
        trace!("listing template files (.{})", template_file_extension);
        let template_files =
        list_path_content(&self.templates_dir())?
            .iter()
            .filter(|p|p.extension()
                        .unwrap_or_else(|| OsStr::new("")) == OsStr::new(template_file_extension)
                        )
            .cloned()
            .collect::<Vec<PathBuf>>();
        ensure!(!template_files.is_empty(), ErrorKind::TemplateNotFound);
        Ok(template_files)
    }

    /// Produces a list of names of all template filess in the `templates_dir()`
    pub fn list_template_names(&self) -> StorageResult<Vec<String>> {
        trace!("listing template names");
        let template_names = self.list_template_files()?.iter()
            .filter_map(|p|p.file_stem())
            .filter_map(OsStr::to_str)
            .map(ToOwned::to_owned)
            .collect();
        Ok(template_names)
    }

    /// Returns the Path to the template file by the given name, maybe.
    pub fn get_template_file(&self, name:&str) -> StorageResult<PathBuf> {
        self.list_template_files()?.iter()
            .filter(|f|f.file_stem().unwrap_or_else(||OsStr::new("")) == name)
            .cloned()
            .nth(0).ok_or_else(||ErrorKind::TemplateNotFound.into())
    }

    /// Produces a list of paths to all archives in the `archive_dir`.
    /// An archive itself is a folder that contains project dirs,
    /// therefore it essentially has the same structure as the `working_dir`,
    /// with the difference, that the project folders may be prefixed with the projects index, e.g.
    /// an invoice number etc.
    pub fn list_archives(&self) -> StorageResult<Vec<PathBuf>> {
        trace!("listing archives files");
        list_path_content(self.archive_dir())
    }

    /// Produces a list of years for which there is an archive.
    pub fn list_years(&self) -> StorageResult<Vec<Year>> {
        trace!("listing years");
        let mut years : Vec<Year> =
            self.list_archives()?
            .iter()
            .filter_map(|p| p.file_stem())
            .filter_map(OsStr::to_str)
            .filter_map(|year_str| year_str.parse::<Year>().ok())
            .collect();
        years.sort();
        Ok(years)
    }

    /// Takes a template file and stores it in the working directory,
    /// in a new project directory according to it's name.
    pub fn create_project(&self, project_name:&str, template_name:&str, fill_data: &HashMap<&str, String>) -> StorageResult<L> {
        debug!("creating a project\n name: {name}\n template: {tmpl}",
               name = project_name,
               tmpl = template_name
               );
        if !self.working_dir().exists(){
            error!("working directory does not exist");
            bail!(ErrorKind::NoWorkingDir)
        };
        let slugged_name = slugify(project_name);
        let project_dir  = self.working_dir().join(&slugged_name);
        if project_dir.exists() {
            error!("project directory already exists");
            bail!(ErrorKind::ProjectDirExists);
        }

        trace!("created project will be called {:?}", slugged_name);

        let target_file  = project_dir
            .join(&(slugged_name + "." + &L::file_extension()));

        let template_path = self.get_template_file(template_name)?;

        trace!("crating project using concrete Project implementation of from_template");
        let mut project = L::from_template(&project_name, &template_path, &fill_data)?;

        // TODO Hand of creation entirely to Storable implementation
        //      Storage it self should only concern itself with Project folders!
        fs::create_dir(&project_dir)?;
        fs::copy(project.file(), &target_file)?;
        trace!("copied project file succesfully");
        project.set_file(&target_file);

        Ok(project)
    }

    /// Moves a project folder from `/working` dir to `/archive/$year`.
    ///
    /// Returns path to new storage dir in archive.
    #[cfg(test)]
    pub fn archive_project_by_name(&self, name:&str, year:Year, prefix:Option<String>) -> StorageResult<PathBuf> {
        info!("archiving project by name {:?} into archive for {}", name, year);
        trace!("prefix {:?}", prefix);

        let slugged_name = slugify(name);
        let name_in_archive = match prefix{
            Some(prefix) => format!("{}_{}", prefix, slugged_name),
                    None => slugged_name
        };

        let archive = self.create_archive(year)?;
        let project_folder = self.get_project_dir(name, StorageDir::Working)?;
        let target = archive.join(&name_in_archive);
        trace!(" moving file into {:?}", target);

        fs::rename(&project_folder, &target)?;

        Ok(target)
    }

    /// Moves a project folder from `/working` dir to `/archive/$year`.
    /// Also adds the project.prefix() to the folder name.
    ///<pre>
    ///└── root
    ///    ├── archive
    ///        ├── 2001
    ///            ├── R0815_Birthdayparty
    ///    ...
    ///</pre>
    // TODO write extra tests
    // TODO make year optional and default to project.year()
    pub fn archive_project(&self, project:&L, year:Year) -> StorageResult<Vec<PathBuf>> {
        debug!("trying archiving {:?} into {:?}", project.short_desc(), year);

        let mut moved_files = Vec::new();

        let name_in_archive = match project.prefix(){
            Some(prefix) => format!("{}_{}", prefix, project.ident()),
            None =>  project.ident()
        };

        let archive = self.create_archive(year)?;
        let project_folder = project.dir();
        let target = archive.join(&name_in_archive);

        fs::rename(&project_folder, &target)?;
        info!("succesfully archived {:?} to {:?}", project.short_desc() ,target);

        moved_files.push(project.dir());
        moved_files.push(target);

        if let Some(repo) = self.repository() {
            repo.add(&moved_files);
        }

        Ok(moved_files)
    }


    /// Moves projects found through `search_terms` from the `Working` directory to the `Archive`/`year` directory.
    ///
    /// Returns list of old and new paths.
    pub fn archive_projects_if<F>(&self, search_terms:&[&str], manual_year:Option<i32>, confirm:F) -> StorageResult<Vec<PathBuf>>
        where F: Fn()->bool
    {
        let projects = self.search_projects_any(StorageDir::Working, search_terms)?;
        let force = confirm();

        ensure!(!projects.is_empty(), ErrorKind:: ProjectDoesNotExist);

        let mut moved_files = Vec::new();

        for project in projects {
            if force {warn!("you are using --force")};
            if project.is_ready_for_archive() || force {
                info!("project {:?} is ready to be archived", project.short_desc());
                let year = manual_year.or_else(|| project.year()).unwrap();
                info!("archiving {} ({})",  project.ident(), project.year().unwrap());
                let mut archive_target = self.archive_project(&project, year)?;
                moved_files.push(project.dir());
                moved_files.append(&mut archive_target);
            }
            else {
                warn!("project {:?} is not ready to be archived", project.short_desc());
            }
        };

        if let Some(repo) = self.repository() {
            repo.add(&moved_files);
        }

        Ok(moved_files)
    }

    pub fn delete_project_if<F>(&self, project:&L, confirmed:F) -> StorageResult<()>
        where F: Fn() -> bool
    {
        debug!("deleting {}", project.dir().display());
        project.delete_project_dir_if(confirmed)?;
        if let Some(ref repo) = self.repository {
            if !repo.add(&[project.dir()]).success() {
                debug!("adding {} to git", project.dir().display());
                bail!(ErrorKind::GitProcessFailed);
            }
        }
        Ok(())
    }


    /// Moves projects found through `search_terms` from the `year` back to the `Working` directory.
    ///
    /// Returns list of old and new paths.
    pub fn unarchive_projects(&self, year:i32, search_terms:&[&str]) -> StorageResult<Vec<(PathBuf)>> {
        let projects = self.search_projects_any(StorageDir::Archive(year), search_terms)?;

        let mut moved_files = Vec::new();
        for project in projects {
            println!("unarchiving {:?}", project.short_desc());
            let unarchive_target = self.unarchive_project(&project).unwrap();
            moved_files.push(project.dir());
            moved_files.push(unarchive_target);
        };

        if let Some(repo) = self.repository() {
            repo.add(&moved_files);
        }

        Ok(moved_files)
    }

    /// Moves a project folder from `/working` dir to `/archive/$year`.
    pub fn unarchive_project(&self, project:&L) -> StorageResult<PathBuf> {
        self.unarchive_project_dir(&project.dir())
    }

    /// Moves a project folder from `/working` dir to `/archive/$year`.
    pub fn unarchive_project_dir(&self, archived_dir:&Path) -> StorageResult<PathBuf> {
        debug!("trying unarchiving {:?}", archived_dir);

        // has to be in archive_dir
        let child_of_archive = archived_dir.starts_with(&self.archive_dir());

        // must not be the archive_dir
        let archive_itself =  archived_dir == self.archive_dir();

        // must be in a dir that parses into a year
        let parent_is_num =  archived_dir.parent()
            .and_then(|p| p.file_stem())
            .and_then(OsStr::to_str)
            .map_or(false, |s| s.parse::<i32>().is_ok());

        let name = self.get_project_name(archived_dir)?;
        let target = self.working_dir().join(&name);
        ensure!(!target.exists(), ErrorKind::ProjectFileExists);
        info!("unarchiving project from {:?} to {:?}", archived_dir, target);

        if child_of_archive && !archive_itself && parent_is_num{
            fs::rename(&archived_dir, &target)?;
        } else {
            error!("moving out of archive failed");
            bail!(ErrorKind::InvalidDirStructure);
        };

        Ok(target.to_owned())
    }

    /// Matches StorageDir's content against a term and returns matching project files.
    ///
    /// This only searches by name
    /// TODO return opened `Project`, no need to reopen
    ///
    /// # Warning
    /// Please be adviced that this uses [`Storage::open_projects()`](struct.Storage.html#method.open_projects) and therefore opens all projects.
    pub fn search_projects(&self, directory:StorageDir, search_term:&str) -> StorageResult<ProjectList<L>> {
        trace!("searching for projects by {:?} in {:?}", search_term, directory);
        let projects = self.open_projects(directory)?
                           .into_iter()
                           .filter(|project| project.matches_search(&search_term.to_lowercase()))
                           .collect();
        Ok(ProjectList{projects})
    }

    /// Matches StorageDir's content against multiple terms and returns matching projects.
    /// TODO add search_multiple_projects_deep
    pub fn search_projects_any(&self, dir:StorageDir, search_terms:&[&str]) -> StorageResult<ProjectList<L>> {
        let mut projects = Vec::new();
        for search_term in search_terms{
            let mut found_projects = self.search_projects(dir, &search_term)?;
            projects.append(&mut found_projects);
        }

        Ok(ProjectList{projects})
    }

    /// Tries to find a concrete Project.
    pub fn get_project_dir(&self, name:&str, directory:StorageDir) -> StorageResult<PathBuf> {
        trace!("getting project directoty for {:?} from {:?}", name, directory);
        let slugged_name = slugify(name);
        if let Ok(path) = match directory {
            StorageDir::Working => Ok(self.working_dir().join(&slugged_name)),
            StorageDir::Archive(year) => self.get_project_dir_from_archive(name, year),
            _ => bail!(ErrorKind::BadChoice)
        }{
            if path.exists(){
                return Ok(path);
            }
        }
        bail!(ErrorKind::ProjectDoesNotExist)
    }

    /// Locates the project file inside a folder.
    ///
    /// This is the first file with the `super::PROJECT_FILE_EXTENSION` in the folder
    pub fn get_project_file(&self, directory:&Path) -> StorageResult<PathBuf> {
        trace!("getting project file from {:?}", directory);
        list_path_content(directory)?.iter()
            .filter(|f|f.extension().unwrap_or_else(||OsStr::new("")) == L::file_extension().as_str())
            .nth(0).map(ToOwned::to_owned)
            .ok_or_else(|| ErrorKind::ProjectDoesNotExist.into())
    }

    fn get_project_name(&self, directory:&Path) -> StorageResult<String> {
        let path = self.get_project_file(directory)?;
        if let Some(stem) = path.file_stem(){
            return Ok(stem.to_str().expect("this filename is no valid unicode").to_owned());
        }
        bail!(ErrorKind::BadProjectFileName)
    }

    fn get_project_dir_from_archive(&self, name:&str, year:Year) -> StorageResult<PathBuf> {
        for project_file in &self.list_project_files(StorageDir::Archive(year))?{
            if project_file.ends_with(slugify(name) + "."+ &L::file_extension()) {
                return project_file.parent().map(|p|p.to_owned()).ok_or_else (|| ErrorKind::ProjectDoesNotExist.into());
            }
        }
        bail!(ErrorKind::ProjectDoesNotExist)
    }

    /// Produces a list of project folders.
    pub fn list_project_folders(&self, directory:StorageDir) -> StorageResult<Vec<PathBuf>> {
        trace!("listing project folders in {:?}-directory", directory);
        match directory{
            StorageDir::Working       => list_path_content(self.working_dir()),
            StorageDir::Archive(year) => {
                let path = self.archive_dir().join(year.to_string());
                let list = list_path_content(&path).unwrap_or_else(|_| Vec::new());
                Ok(list)
            },
            StorageDir::All           => {
                let mut all:Vec<PathBuf> = Vec::new();
                for year in self.list_years()?{
                    all.append(&mut list_path_content(&self.archive_dir().join(year.to_string()))?);
                }
                all.append(&mut list_path_content(&self.working_dir())?);
                Ok(all)
            },
            _ => bail!(ErrorKind::BadChoice)
        }
    }

    /// Produces a list of empty project folders.
    pub fn list_empty_project_dirs(&self, directory:StorageDir) -> StorageResult<Vec<PathBuf>> {
        trace!("listing empty project dirs {:?}-directory", directory);
        let projects = self.list_project_folders(directory)?.iter()
            .filter(|dir| self.get_project_file(dir).is_err())
            .cloned()
            .collect();
        Ok(projects)
    }

    /// Produces a list of project files.
    pub fn list_project_files(&self, directory:StorageDir) -> StorageResult<Vec<PathBuf>> {
        trace!("listing project files in {:?}-directory", directory);
        self.list_project_folders(directory)?.iter()
            .map(|dir| self.get_project_file(dir))
            .collect()
    }

    pub fn filter_project_files<F>(&self, directory:StorageDir, filter:F) -> StorageResult<Vec<PathBuf>>
        where F:FnMut(&PathBuf) -> bool
    {
        trace!("filtering project files in {:?}-directory", directory);
        let projects = self.list_project_folders(directory)?.iter()
            .filter_map(|dir| self.get_project_file(dir).ok())
            .filter(filter)
            .collect();
        Ok(projects)
    }

    /// Behaves like `list_project_files()` but also opens projects directly.
    pub fn open_projects<I>(&self, selection:I) -> StorageResult<ProjectList<L>>
        where I: Into<StorageSelection>
    {
        use self::StorageSelection::*;
        let projects = match selection.into() {
            DirAndSearch(dir, ref search_terms) => {
                let terms = search_terms.iter().map(AsRef::as_ref).collect::<Vec<_>>(); // sorry about this
                let projects = self.search_projects_any(dir, &terms)?;
                if projects.is_empty() {
                    return Err(format!("Nothing found for {:?}", search_terms).into())
                }
                projects
            },
            Dir(dir) => self.open_projects_dir(dir)?,
            Paths(ref paths) => self.open_paths(paths),
            Unintiailzed => unreachable!()
        };
        Ok(projects)
    }

    fn open_paths(&self, paths: &[PathBuf]) -> ProjectList<L> {
        let mut projects = paths.par_iter()
            .filter_map(|path| Self::open_project(path).ok())
            .collect::<Vec<L>>();

        if cfg!(feature="git_statuses") {
            if let Some(ref repo) = self.repository {
                return projects
                    .drain(..)
                    .map(|mut project| {
                        let dir = project.dir();
                        project.set_git_status(repo.get_status(&dir));
                        project
                    })
                    .collect();
            }
        }

        ProjectList {
            projects
        }
    }

    /// Behaves like `list_project_files()` but also opens projects directly.
    pub fn open_projects_dir(&self, directory:StorageDir) -> StorageResult<ProjectList<L>>{
        debug!("OPENING ALL PROJECTS in {:?}-directory", directory);
        match directory {
            StorageDir::Year(year) => {
                // recursive :D
                let mut archived = self.open_projects(StorageDir::Archive(year))?;
                let mut working = self.open_projects(StorageDir::Working)?;
                archived.append(working.deref_mut());
                archived.filter_by_key_val("Year", year.to_string().as_ref());
                Ok(archived)
            },
            _ =>
                self.list_project_folders(directory)
                .map(|p| self.open_paths(&p))
        }
    }

    pub fn open_working_dir_projects(&self) -> StorageResult<ProjectList<L>> {
        debug!("OPENING ALL WORKING DIR PROJECTS");
        Ok(self.open_projects(StorageDir::Working)?)
    }

    pub fn open_all_archived_projects(&self) -> StorageResult<ProjectsByYear<L>> {
        debug!("OPENING ALL ARCHIVED PROJECTS");
        let mut map = LinkedHashMap::new();
        for year in self.list_years()? {
            map.insert(year, self.open_projects(StorageDir::Archive(year))?);
        }
        Ok(map)
    }

    pub fn open_all_projects(&self) -> StorageResult<Projects<L>> {
        debug!("OPENING ALL PROJECTS");
        Ok( Projects {
            working: self.open_projects(StorageDir::Working)?,
            archive: self.open_all_archived_projects()?
        })
    }

    fn open_project(path: &PathBuf) -> StorageResult<L> {
        let project = L::open_folder(path);
        if let Err(ref err) = project {
            warn!("{}", err);
        }
        project
    }

}

impl<P:Storable> fmt::Debug for Storage<P>{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "Storage: storage  = {storage:?}
                          working  = {working:?}
                          archive  = {archive:?}
                          template = {template:?}",
               storage  = self.root_dir(),
               working  = self.working_dir(),
               archive  = self.archive_dir(),
               template = self.templates_dir(),
               )
    }
}
