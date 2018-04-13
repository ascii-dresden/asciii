#[cfg(feature="localization")]
include!(concat!(env!("OUT_DIR"), "/localize_macros.rs"));

#[cfg(not(feature="localization"))]
include!("./localize_macros_static.rs");
