use super::*;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links { }

    foreign_links {
        Io(io::Error);
        Fmt(fmt::Error);
        Time(time::SystemTimeError);
        Handlebar(RenderError);
        Project(project::error::Error);
        Storage(StorageError);
    }

    errors {
        NoPdfCreated{ description("No Pdf Created") }
        NothingToDo{ description("Nothing to do") }
    }
}
