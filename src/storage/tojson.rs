use rustc_serialize::json::{ToJson, Json};

use std::path::Path;

use super::storable::Storable;
use super::Storage;

impl<P:Storable> ToJson for Storage<P>{
    fn to_json(&self) -> Json{
        let s = |s:&str| String::from(s);
        let p = |p:&Path| p.display().to_string().to_json();
        Json::Object(btreemap!{
            s("dirs") =>
                Json::Object(btreemap!{
                    s("storage") => p(self.root_dir()),
                    s("working")  => p(self.working_dir()),
                    s("archive")  => p(self.archive_dir()),
                    s("template") => p(self.templates_dir()),
                })
        })
    }
}
