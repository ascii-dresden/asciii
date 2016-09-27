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


use std::fs;
use std::path::{Path, PathBuf};
use std::marker::PhantomData;

/// Year = `i32`
pub type Year =  i32;

/// Result returned by Storage
pub type StorageResult<T> = Result<T, StorageError>;

#[cfg(test)] mod test;
#[cfg(test)] mod realworld;

mod project_list;
pub use self::project_list::ProjectList;
pub mod repo;
pub mod error;
pub use self::error::StorageError;
pub mod storable;
pub use self::storable::Storable;

#[cfg(feature="document_export")]
mod tojson;

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

    project_type: PhantomData<L>,

    pub repository: Option<Repository>
}

/// Used to identify what directory you are talking about.
#[derive(Debug,Clone,Copy)]
#[allow(dead_code)]
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
    /// `Archive` and `Working` directory, not `Templates`.
    All
}

/// Basically `ls`, returns a list of paths.
pub fn list_path_content(path:&Path) -> StorageResult<Vec<PathBuf>> {
    Ok(try!(fs::read_dir(path))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .collect::<Vec<PathBuf>>())
}



use self::repo::Repository;

use std::fmt;
use std::ffi::OsStr;
use std::ops::DerefMut;
use std::collections::HashMap;

use slug;
fn slugify(string:&str) -> String{ slug::slugify(string) }


impl<L:Storable> Storage<L> {
    /// Inits storage, does not check existence, yet. TODO
    pub fn new<P: AsRef<Path>>(root:P, working:&str, archive:&str, template:&str) -> StorageResult<Self> {
        trace!("initializing storage, root: {}", root.as_ref().display());
        let root = root.as_ref();
        if root.is_absolute(){
            Ok( Storage{
                root:   root.to_path_buf(),
                working:   root.join(working),
                archive:   root.join(archive),
                templates: root.join(template),
                project_type: PhantomData,
                repository: None,
            })
        } else {
            Err(StorageError::StoragePathNotAbsolute)
        }
    }

    /// Inits storage with git capabilities.
    pub fn new_with_git<P: AsRef<Path>>(root:P, working:&str, archive:&str, template:&str) -> StorageResult<Self> {
        trace!("initializing storage, with git");
        Ok( Storage{
            repository: Some(try!(Repository::new(root.as_ref()))),
            .. try!{Self::new(root,working,archive,template)}
        })
    }

    /// Getter for Storage::storage.
    pub fn root_dir(&self) -> &Path{
        self.root.as_ref()
    }

    /// Getter for Storage::working.
    pub fn working_dir(&self) -> &Path{
        self.working.as_ref()
    }

    /// Getter for Storage::archive.
    pub fn archive_dir(&self) -> &Path{
        self.archive.as_ref()
    }

    /// Getter for Storage::templates.
    pub fn templates_dir(&self) -> &Path{
        self.templates.as_ref()
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
        if !self.root_dir().is_absolute() { return Err(StorageError::StoragePathNotAbsolute) }

        if !self.root_dir().exists()  { try!(fs::create_dir(&self.root_dir()));  }
        if !self.working_dir().exists()  { try!(fs::create_dir(&self.working_dir()));  }
        if !self.archive_dir().exists()  { try!(fs::create_dir(&self.archive_dir()));  }
        if !self.templates_dir().exists() { try!(fs::create_dir(&self.templates_dir())); }

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
            try!(fs::create_dir(archive));
        }
        Ok(archive.to_owned())
    }

    /// Produces a list of files in the `template_dir()`
    pub fn list_template_files(&self) -> StorageResult<Vec<PathBuf>> {
        let template_file_extension = ::CONFIG.get_str("extensions/project_template").expect("Internal Error: default config is wrong");
        trace!("listing template files (.{})", template_file_extension);
        let template_files :Vec<PathBuf>=
        try!(list_path_content(&self.templates_dir()))
            .iter()
            .filter(|p|p.extension()
                        .unwrap_or_else(|| OsStr::new("")) == OsStr::new(template_file_extension)
                        )
            .cloned()
            .collect();
        if template_files.is_empty(){
            Err(StorageError::TemplateNotFound) // TODO: RFC perhaps "NoTemplates"?
        } else {
            Ok(template_files)
        }
    }

    /// Produces a list of names of all template filess in the `templates_dir()`
    pub fn list_template_names(&self) -> StorageResult<Vec<String>> {
        trace!("listing template names");
        let template_names = try!(self.list_template_files()).iter()
            .filter_map(|p|p.file_stem())
            .filter_map(OsStr::to_str)
            .map(ToOwned::to_owned)
            .collect();
        Ok(template_names)
    }

    /// Returns the Path to the template file by the given name, maybe.
    pub fn get_template_file(&self, name:&str) -> StorageResult<PathBuf> {
        try!(self.list_template_files()).iter()
            .filter(|f|f.file_stem().unwrap_or_else(||OsStr::new("")) == name)
            .cloned()
            .nth(0).ok_or(StorageError::TemplateNotFound)
    }

    /// Produces a list of paths to all archives in the `archive_dir`.
    /// An archive itself is a folder that contains project dirs,
    /// therefore it essentially has the same structure as the `working_dir`,
    /// with the difference, that the project folders may be prefixed with the projects index, e.g.
    /// an invoice number etc.
    pub fn list_archives(&self) -> StorageResult<Vec<PathBuf>> {
        list_path_content(self.archive_dir())
    }

    /// Produces a list of years for which there is an archive.
    pub fn list_years(&self) -> StorageResult<Vec<Year>> {
        trace!("listing years");
        let mut years : Vec<Year> =
            try!(self.list_archives())
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
    pub fn create_project(&self, project_name:&str, template_name:&str, fill_data:&HashMap<&str, String>) -> StorageResult<L> {
        debug!("creating a project\n name: {name}\n template: {tmpl}",
               name = project_name,
               tmpl = template_name
               );
        if !self.working_dir().exists(){
            error!("working directory does not exist");
            return Err(StorageError::NoWorkingDir)
        };
        let slugged_name = slugify(project_name);
        let project_dir  = self.working_dir().join(&slugged_name);
        if project_dir.exists() {
            error!("project directory already exists");
            return Err(StorageError::ProjectDirExists);
        }

        trace!("created project will be called {:?}", slugged_name);

        let target_file  = project_dir
            .join(&(slugged_name + "." + L::file_extension()));

        let template_path = try!(self.get_template_file(template_name));

        trace!("crating project using concrete Project implementation of from_template");
        let mut project = try!(L::from_template(&project_name, &template_path, &fill_data));

        // TODO Hand of creation entirely to Storable implementation
        //      Storage it self should only concern itself with Project folders!
        try!(fs::create_dir(&project_dir));
        try!(fs::copy(project.file(), &target_file));
        trace!("copied project file succesfully");
        project.set_file(&target_file);

        Ok(project)
    }

    /// Moves a project folder from `/working` dir to `/archive/$year`.
    ///
    /// Returns path to new storage dir in archive.
    pub fn archive_project_by_name(&self, name:&str, year:Year, prefix:Option<String>) -> StorageResult<PathBuf> {
        info!("archiving project by name {:?} into archive for {}", name, year);
        trace!("prefix {:?}", prefix);

        let slugged_name = slugify(name);
        let name_in_archive = match prefix{
            Some(prefix) => format!("{}_{}", prefix, slugged_name),
                    None => slugged_name
        };

        let archive = try!(self.create_archive(year));
        let project_folder = try!(self.get_project_dir(name, StorageDir::Working));
        let target = archive.join(&name_in_archive);
        trace!(" moving file into {:?}", target);

        try!(fs::rename(&project_folder, &target));

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
    pub fn archive_project(&self, project:&L, year:Year) -> StorageResult<PathBuf> {
        debug!("trying archiving {:?} into {:?}", project.name(), year);
        let name_in_archive = match project.prefix(){
            Some(prefix) => format!("{}_{}", prefix, project.ident()),
            None =>  project.ident()
        };

        let archive = try!(self.create_archive(year));
        let project_folder = project.dir();
        let target = archive.join(&name_in_archive);

        try!(fs::rename(&project_folder, &target));
        info!("succesfully archived {:?} to {:?}", project.name() ,target);

        Ok(target)
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

        let name = try!(self.get_project_name(archived_dir));
        let target = self.working_dir().join(&name);
        if target.exists() { return Err(StorageError::ProjectFileExists); }
        info!("unarchiving project from {:?} to {:?}", archived_dir, target);

        if child_of_archive && !archive_itself && parent_is_num{
            try!(fs::rename(&archived_dir, &target));
        }else{
            error!("moving out of archive failed");
            return Err(StorageError::InvalidDirStructure);
        };

        Ok(target.to_owned())
    }

    pub fn delete_project(&self, project:&L) -> StorageResult<PathBuf> {
        self.delete_project_dir(&project.dir())
    }

    pub fn delete_project_dir(&self, folder:&Path) -> StorageResult<PathBuf> {
        debug!("$ rm {}", folder.display());
        match fs::remove_dir_all(folder){
            Ok(_) => Ok(folder.to_owned()),
            Err(_) => Err(StorageError::ProjectDoesNotExist)
        }
    }

    /// Matches StorageDir's content against a term and returns matching project files.
    ///
    /// This only searches by name
    /// TODO return opened `Project`, no need to reopen
    ///
    /// # Warning
    /// Please be adviced that this uses [`Storage::open_projects()`](struct.Storage.html#method.open_projects) and therefore opens all projects.
    pub fn search_projects(&self, directory:StorageDir, search_term:&str) -> StorageResult<Vec<L>> {
        trace!("searching for projects by {:?} in {:?}", search_term, directory);
        let project_paths = try!(self.open_projects(directory));
        let projects = project_paths
            .into_iter()
            .filter(|project| project.matches_search(&search_term.to_lowercase()))
            .collect();
        Ok(projects)
    }

    /// Matches StorageDir's content against multiple terms and returns matching project files.
    /// TODO add search_multiple_projects_deep
    pub fn search_projects_any(&self, dir:StorageDir, search_terms:&[&str]) -> StorageResult<Vec<L>> {
        let mut all_paths = Vec::new();
        for search_term in search_terms{
            let mut paths = try!(self.search_projects(dir, &search_term));
            all_paths.append(&mut paths);
        }

        Ok(all_paths)
    }

    /// Tries to find a concrete Project.
    pub fn get_project_dir(&self, name:&str, directory:StorageDir) -> StorageResult<PathBuf> {
        trace!("getting project directoty for {:?} from {:?}", name, directory);
        let slugged_name = slugify(name);
        if let Ok(path) = match directory{
            StorageDir::Working => Ok(self.working_dir().join(&slugged_name)),
            StorageDir::Archive(year) => self.get_project_dir_from_archive(name, year),
            _ => return Err(StorageError::BadChoice)
        }{
            if path.exists(){
                return Ok(path);
            }
        }
        Err(StorageError::ProjectDoesNotExist)
    }

    /// Locates the project file inside a folder.
    ///
    /// This is the first file with the `super::PROJECT_FILE_EXTENSION` in the folder
    pub fn get_project_file(&self, directory:&Path) -> StorageResult<PathBuf> {
        trace!("getting project file from {:?}", directory);
        try!(list_path_content(directory)).iter()
            .filter(|f|f.extension().unwrap_or_else(||OsStr::new("")) == L::file_extension())
            .nth(0).map(ToOwned::to_owned)
            .ok_or(StorageError::ProjectDoesNotExist)
    }

    fn get_project_name(&self, directory:&Path) -> StorageResult<String> {
        let path = try!(self.get_project_file(directory));
        if let Some(stem) = path.file_stem(){
            return Ok(stem.to_str().expect("this filename is no valid unicode").to_owned());
        }
        Err(StorageError::BadProjectFileName)
    }

    fn get_project_dir_from_archive(&self, name:&str, year:Year) -> StorageResult<PathBuf> {
        for project_file in &try!(self.list_project_files(StorageDir::Archive(year))){
            if project_file.ends_with(slugify(name) + "."+ L::file_extension()) {
                return project_file.parent().map(|p|p.to_owned()).ok_or(StorageError::ProjectDoesNotExist);
            }
        }
        Err(StorageError::ProjectDoesNotExist)
    }

    /// Produces a list of project folders.
    pub fn list_project_folders(&self, directory:StorageDir) -> StorageResult<Vec<PathBuf>> {
        trace!("listing project folders in {:?}-directory", directory);
        match directory{
            StorageDir::Working       => list_path_content(self.working_dir()),
            StorageDir::Archive(year) => list_path_content(&self.archive_dir().join(year.to_string())),
            StorageDir::All           => {
                let mut all:Vec<PathBuf> = Vec::new();
                for year in try!(self.list_years()){
                    all.append(&mut try!(list_path_content(&self.archive_dir().join(year.to_string()))));
                }
                all.append(&mut try!(list_path_content(&self.working_dir())));
                Ok(all)
            },
            _ => Err(StorageError::BadChoice)
        }
    }

    /// Produces a list of empty project folders.
    pub fn list_empty_project_dirs(&self, directory:StorageDir) -> StorageResult<Vec<PathBuf>> {
        trace!("listing empty project dirs {:?}-directory", directory);
        let projects = try!(self.list_project_folders(directory)).iter()
            .filter(|dir| self.get_project_file(dir).is_err())
            .cloned()
            .collect();
        Ok(projects)
    }

    /// Produces a list of project files.
    pub fn list_project_files(&self, directory:StorageDir) -> StorageResult<Vec<PathBuf>> {
        trace!("listing project files in {:?}-directory", directory);
        try!(self.list_project_folders(directory)).iter()
            .map(|dir| self.get_project_file(dir))
            .collect()
    }

    pub fn filter_project_files<F>(&self, directory:StorageDir, filter:F) -> StorageResult<Vec<PathBuf>>
        where F:FnMut(&PathBuf) -> bool
    {
        trace!("filtering project files in {:?}-directory", directory);
        let projects = try!(self.list_project_folders(directory)).iter()
            .filter_map(|dir| self.get_project_file(dir).ok())
            .filter(filter)
            .collect();
        Ok(projects)
    }

    /// Behaves like `list_project_files()` but also opens projects directly.
    pub fn open_projects(&self, directory:StorageDir) -> StorageResult<ProjectList<L>>{
        trace!("OPENING ALL PROJECTS in {:?}-directory", directory);
        match directory {
            StorageDir::Year(year) => {
                // recursive :D
                let mut archived = try!(self.open_projects(StorageDir::Archive(year)));
                let mut working = try!(self.open_projects(StorageDir::Working));
                archived.append(working.deref_mut());
                archived.filter_by_key_val("Year", year.to_string().as_ref());
                Ok(archived)
            },
            _ =>
                self.list_project_folders(directory)
                .map(|paths| ProjectList{
                    projects:paths.iter()
                        .filter_map(|path|self.open_project(path))
                        .collect::<Vec<L>>()
                }
                )
        }
    }

    #[cfg(not(feature="git_statuses"))]
    fn open_project(&self, path:&PathBuf) -> Option<L>{
        match L::open(path) {
            Ok(project) => Some(project) ,
            Err(err) => {
                warn!("Erroneous Project: {}\n {:#?}", path.display(), err);
                None
            }
        }
    }


    #[cfg(feature="git_statuses")]
    fn open_project(&self, path:&PathBuf) -> Option<L>{
        match L::open(path) {
            Ok(mut project) => {
                if let Some(ref repo) = self.repository{
                    project.set_git_status(repo.get_status(path));
                }
                Some(project) // "return"
            },
            Err(err) => {
                warn!("Erroneous Project: {}\n {:#?}", path.display(), err);
                None
            }
        }
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

#[derive(Debug)]
#[deprecated(note="was a bad idea, settle for `StorageDir` + search_terms:&[&str]")]
pub struct Selection<'a>{
    pub search: &'a str,
    pub dir: StorageDir,
}

impl<'a> Selection<'a>{
    pub fn new(search:&'a str,archive:Option<i32>) -> Self{
        Selection{
            search: search,
            dir: match archive{
                Some(year) => StorageDir::Archive(year),
                None => StorageDir::Working
            }
        }
    }
}
