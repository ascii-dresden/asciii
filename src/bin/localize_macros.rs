#[cfg(feature="localize")]
include!(concat!(env!("OUT_DIR"), "/localize_macros.rs"));

#[cfg(not(feature="localize"))]
include!("../localize_macros_static.rs");
