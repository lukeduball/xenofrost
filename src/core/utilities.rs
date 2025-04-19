#[macro_export]
macro_rules! include_str_from_project_path {
    ($string:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), $string))
    };
}

#[macro_export]
macro_rules! include_bytes_from_project_path {
    ($string:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), $string))
    };
}

pub(crate) use include_str_from_project_path;