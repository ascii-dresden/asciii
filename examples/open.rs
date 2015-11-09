extern crate ascii_invoicer;
use ascii_invoicer::project::Project;

fn main(){
    let p = Project::from_yaml_file("./test/template1.yml");
    println!("{:?}", p.created());
    p.filter_all();
    println!("{:?}", p.manager());
}
