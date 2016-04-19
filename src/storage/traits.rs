//! Reimplementing Storage as trait
#![cfg(feature="new_storage")]

#![allow(dead_code)]

use std::fs;
use std::fmt;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::marker::PhantomData;

use slug;

use repo::Repository;

use super::error::StorageError;
use super::Year;
use super::StorageDir;
use super::StorageResult;
use super::ProjectList;
use super::storable::Storable;

/// TODO rename this into Storage later
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
}

/// This will add git functionality on top of Storage
pub struct GitStorage<L:Storable> {
    storage: self::Storage<L>,
    pub repository: Repository
}

/// Base functionality for archiving and unarchiving etc
pub trait Storing<L:Storable>: Sized{
    fn new<P: AsRef<Path>>(root:P, working:&str, archive:&str, template:&str) -> StorageResult<Self>;
    /// Getter path to root directory.
    fn root_dir(&self) -> &Path;
    /// Getter path to working directory.
    fn working_dir(&self) -> &Path;
    /// Getter path to archive directory.
    fn archive_dir(&self) -> &Path;
    /// Getter path to templates directory.
    fn templates_dir(&self) -> &Path;

    // COPY PAST FROM old `struct Storage`

    /// Generic Filesystem wrapper.
    fn list_path_content(&self, path:&Path) -> StorageResult<Vec<PathBuf>> {
        let entries = try!(fs::read_dir(path))
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect::<Vec<PathBuf>>();
        Ok(entries)
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
    fn create_dirs(&self) -> StorageResult<()> {
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
    fn create_archive(&self, year:Year) -> StorageResult<PathBuf> {
        assert!(self.archive_dir().exists());
        let archive = &self.archive_dir().join(year.to_string());

        if self.archive_dir().exists() && !archive.exists() {
            try!(fs::create_dir(archive));
        }
        Ok(archive.to_owned())
    }



    /// Produces a list of files in the `template_dir()`
    fn list_template_files(&self) -> StorageResult<Vec<PathBuf>> {
        let template_files :Vec<PathBuf>=
        try!(self.list_path_content(&self.templates_dir()))
            .iter()
            .filter(|p|p.extension()
                        .unwrap_or_else(|| OsStr::new("")) == OsStr::new(super::TEMPLATE_FILE_EXTENSION)
                        )
            .cloned().collect();
        if template_files.is_empty(){
            Err(StorageError::TemplateNotFound) // TODO: RFC perhaps "NoTemplates"?
        } else {
            Ok(template_files)
        }
    }

    /// Produces a list of names of all template filess in the `templates_dir()`
    fn list_template_names(&self) -> StorageResult<Vec<String>> {
        let template_names =
        try!(self.list_template_files()).iter()
            .filter_map(|p|p.file_stem())
            .filter_map(|n|n.to_str())
            .map(|s|s.to_owned())
            .collect();
        Ok(template_names)
    }

    /// Returns the Path to the template file by the given name, maybe.
    fn get_template_file(&self, name:&str) -> StorageResult<PathBuf> {
        try!(self.list_template_files()).iter()
            .filter(|f|f.file_stem().unwrap_or(&OsStr::new("")) == name)
            .cloned()
            .nth(0).ok_or(StorageError::TemplateNotFound)
    }

    /// Produces a list of paths to all archives in the `archive_dir`.
    /// An archive itself is a folder that contains project dirs,
    /// therefore it essentially has the same structure as the `working_dir`,
    /// with the difference, that the project folders may be prefixed with the projects index, e.g.
    /// an invoice number etc.
    fn list_archives(&self) -> StorageResult<Vec<PathBuf>> {
        self.list_path_content(&self.archive_dir())
    }

    /// Produces a list of years for which there is an archive.
    fn list_years(&self) -> StorageResult<Vec<Year>> {
        let mut years : Vec<Year> =
            try!(self.list_archives())
            .iter()
            .filter_map(|p| p.file_stem())
            .filter_map(|p| p.to_str())
            .filter_map(|year_str| year_str.parse::<Year>().ok())
            .collect();
        years.sort();
        Ok(years)
    }

    /// Takes a template file and stores it in the working directory,
    /// in a new project directory according to it's name.
    fn create_project(&self, project_name:&str, template_name:&str) -> StorageResult<L> {
        if !self.working_dir().exists(){ return Err(StorageError::NoWorkingDir)}; // funny syntax
        let slugged_name = slug::slugify(&project_name);
        let project_dir  = self.working_dir().join(&slugged_name);
        if project_dir.exists() { return Err(StorageError::ProjectDirExists); }

        let target_file  = project_dir
            .join(&(slugged_name + "." + super::PROJECT_FILE_EXTENSION));

        let template_path = try!(self.get_template_file(template_name));
        let mut project = try!(L::from_template(&project_name, &template_path));

        // TODO test for unreplaced template keywords
        try!(fs::create_dir(&project_dir));
        try!(fs::copy(project.file(), &target_file));
        project.set_file(&target_file);

        Ok(project)
    }

    /// Moves a project folder from `/working` dir to `/archive/$year`.
    fn archive_project_by_name(&self, name:&str, year:Year, prefix:Option<String>) -> StorageResult<PathBuf> {
        let slugged_name = slug::slugify(name);
        let name_in_archive = match prefix{
            Some(prefix) => format!("{}_{}", prefix, slugged_name),
                    None => slugged_name
        };

        let archive = try!(self.create_archive(year));
        let project_folder = try!(self.get_project_dir(name, StorageDir::Working));
        let target = archive.join(&name_in_archive);

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
    fn archive_project(&self, project:&L, year:Year) -> StorageResult<PathBuf> {
        let name_in_archive = match project.prefix(){
            Some(prefix) => format!("{}_{}", prefix, project.ident()),
                    None =>  project.ident()
        };

        let archive = try!(self.create_archive(year));
        let project_folder = project.dir();
        let target = archive.join(&name_in_archive);

        try!(fs::rename(&project_folder, &target));

        Ok(target)
    }

    /// Moves a project folder from `/working` dir to `/archive/$year`.
    fn unarchive_project(&self, project:&L) -> StorageResult<PathBuf> {
        self.unarchive_project_file(&project.file())
    }

    /// Moves a project folder from `/working` dir to `/archive/$year`.
    fn unarchive_project_file(&self, archived_file:&Path) -> StorageResult<PathBuf> {
        let archived_dir = if archived_file.is_file() { try!(archived_file.parent().ok_or(StorageError::InvalidDirStructure)) } else {archived_file};

        // has to be in archive_dir
        let child_of_archive = archived_file.starts_with(&self.archive_dir());

        // must not be the archive_dir
        let archive_itself =  archived_dir == self.archive_dir();

        // must be in a dir that parses into a year
        let parent_is_num =  archived_dir.parent()
            .and_then(|p| p.file_stem())
            .and_then(|p| p.to_str())
            .map(|s| s.parse::<i32>().is_ok() )
            .unwrap_or(false);

        let name = try!(self.get_project_name(archived_dir));
        let target = self.working_dir().join(&name);
        if target.exists() { return Err(StorageError::ProjectFileExists); }

        if child_of_archive && !archive_itself && parent_is_num{
            try!(fs::rename(&archived_dir, &target));
        }else{
            println!("not cool");
            return Err(StorageError::InvalidDirStructure);
        };

        Ok(target.to_owned())
    }

    /// Matches StorageDir's content against a term and returns matching project files.
    ///
    /// This only searches by name
    /// TODO return opened `Project`, no need to reopen
    /// TODO rename `to search_by_name`
    pub fn search_projects(&self, dir:StorageDir, search_term:&str) -> StorageResult<Vec<PathBuf>> {
        let project_paths = try!(self.open_project_files(dir)).iter()
            .filter(|project| project.matches_search(&search_term.to_lowercase()))
            .map(|project| project.file())
            .collect();
        Ok(project_paths)
    }

    /// Matches StorageDir's content against a term and returns matching project files.
    /// TODO add search_multiple_projects_deep
    fn search_multiple_projects(&self, dir:StorageDir, search_terms:&[&str]) -> StorageResult<Vec<PathBuf>> {
        let mut all_paths = Vec::new();
        for search_term in search_terms{
            let mut paths = try!(self.search_projects(dir.clone(), &search_term));
            all_paths.append(&mut paths);
        }

        Ok(all_paths)
    }

    /// Tries to find a concrete Project.
    fn get_project_dir(&self, name:&str, directory:StorageDir) -> StorageResult<PathBuf> {
        let slugged_name = slug::slugify(name);
        if let Ok(path) = match directory{
            StorageDir::Working => Ok(self.working_dir().join(&slugged_name)),
            StorageDir::Archive(year) => self.get_project_dir_from_archive(&name, year),
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
    fn get_project_file(&self, directory:&Path) -> StorageResult<PathBuf> {
        try!(self.list_path_content(directory)).iter()
            .filter(|f|f.extension().unwrap_or(&OsStr::new("")) == super::PROJECT_FILE_EXTENSION)
            .nth(0).map(|b|b.to_owned())
            .ok_or(StorageError::ProjectDoesNotExist)
    }

    // TODO I'm redundant, make me generic
    fn get_project_name(&self, directory:&Path) -> StorageResult<String> {
        let path = try!(self.get_project_file(directory));
        Ok(path.file_stem().unwrap().to_str().unwrap().to_owned())
    }

    fn get_project_dir_from_archive(&self, name:&str, year:Year) -> StorageResult<PathBuf> {
        for project_file in &try!(self.list_project_files(StorageDir::Archive(year))){
            if project_file.ends_with(slug::slugify(&name) + "."+ super::PROJECT_FILE_EXTENSION) {
                return project_file.parent().map(|p|p.to_owned()).ok_or(StorageError::ProjectDoesNotExist);
            }
        }
        Err(StorageError::ProjectDoesNotExist)
    }

    /// Produces a list of project folders.
    fn list_project_folders(&self, directory:StorageDir) -> StorageResult<Vec<PathBuf>> {
        match directory{
            StorageDir::Working       => self.list_path_content(&self.working_dir()),
            StorageDir::Archive(year) => self.list_path_content(&self.archive_dir().join(year.to_string())),
            StorageDir::All           => {
                let mut all:Vec<PathBuf> = Vec::new();
                for year in try!(self.list_years()){
                    all.append(&mut try!(self.list_path_content(&self.archive_dir().join(year.to_string()))));
                }
                all.append(&mut try!(self.list_path_content(&self.working_dir())));
                Ok(all)
            },
            _ => Err(StorageError::BadChoice)
        }
    }

    /// Produces a list of empty project folders.
    fn list_empty_project_dirs(&self, directory:StorageDir) -> StorageResult<Vec<PathBuf>> {
        let projects = try!(self.list_project_folders(directory)).iter()
            .filter(|dir| self.get_project_file(dir).is_err())
            .cloned()
            .collect();
        Ok(projects)
    }

    /// Produces a list of project files.
    fn list_project_files(&self, directory:StorageDir) -> StorageResult<Vec<PathBuf>> {
        let projects = try!(self.list_project_folders(directory)).iter()
            .filter_map(|dir| self.get_project_file(dir).ok())
            .collect();
        Ok(projects)
    }

    fn filter_project_files<F>(&self, directory:StorageDir, filter:F) -> StorageResult<Vec<PathBuf>>
        where F:FnMut(&PathBuf) -> bool
    {
        let projects = try!(self.list_project_folders(directory)).iter()
            .filter_map(|dir| self.get_project_file(dir).ok())
            .filter(filter)
            .collect();
        Ok(projects)
    }

    /// Behaves like `list_project_files()` but also opens projects directly.
    fn open_project_files(&self, dir:StorageDir) -> StorageResult<ProjectList<L>>{
        self.list_project_files(dir)
            .map(|paths| ProjectList{projects:paths.iter()
                 .filter_map(|path| match L::open(path){
                     Ok(project) => Some(project),
                     Err(err) => {
                         println!("Erroneous Project: {}\n {:#?}", path.display(), err);
                         None
                     }
                 }).collect::<Vec<L>>()}
                )
    }
}


impl<L:Storable> Storing<L> for Storage<L> {
    fn new<P: AsRef<Path>>(root:P, working:&str, archive:&str, template:&str) -> StorageResult<Self> {
        let root = root.as_ref();
        if root.is_absolute(){
            Ok( Storage{
                root:      root.to_path_buf(),
                working:   root.join(working),
                archive:   root.join(archive),
                templates: root.join(template),
                project_type: PhantomData,
            })
        } else {
            Err(StorageError::StoragePathNotAbsolute)
        }
    }
    fn root_dir(&self)      -> &Path{ self.root.as_ref() }
    fn working_dir(&self)   -> &Path{ self.working.as_ref() }
    fn archive_dir(&self)   -> &Path{ self.archive.as_ref() }
    fn templates_dir(&self) -> &Path{ self.templates.as_ref() }
}

impl<L:Storable> Storing<L> for GitStorage<L> {
    fn new<P: AsRef<Path>>(root:P, working:&str, archive:&str, template:&str) -> StorageResult<Self> {
        Ok(GitStorage{
            storage:  try!{Storage::new(root.as_ref(),working,archive,template)},
            repository: try!(Repository::new(&root.as_ref()))
        })
    }

    fn root_dir(&self)      -> &Path{ self.storage.root_dir() }
    fn working_dir(&self)   -> &Path{ self.storage.working_dir() }
    fn archive_dir(&self)   -> &Path{ self.storage.archive_dir() }
    fn templates_dir(&self) -> &Path{ self.storage.templates_dir() }
}

