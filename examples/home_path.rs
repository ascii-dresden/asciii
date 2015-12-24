use std::path::Path;
use std::path::PathBuf;
//use std::fs::Metadata;
//use std::ffi::OsString;

fn replace_home_tilde(p:&Path) -> PathBuf{
    let path = p.to_str().unwrap();
    PathBuf::from( path.replace("~",std::env::home_dir().unwrap().to_str().unwrap()))
}

fn main(){
    let p = Path::new("~/foo.txt");
    println!("{:?}, {}", p, p.display());
    println!("{}", p.exists());
    println!("{}", p.is_absolute());
    println!("{}", p.has_root());
    println!("{:?}", p.components().next());

    let p = Path::new("~/foo.txt");
    let p = replace_home_tilde(p);
    println!("{:?}, {}", p, p.display());
    println!("{}", p.exists());
    println!("{}", p.is_absolute());
    println!("{}", p.has_root());
    println!("{:?}", p.components().next());
}
