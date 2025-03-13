use xenofrost_macros::{get_number_of_resources, get_number_of_components};

const BASELINE_NUMBER_OF_RESOURCES: u64 = 0;
const BASELINE_NUMBER_OF_COMPONENTS: u64 = 0;

pub mod core;

pub const NUMBER_OF_RESOURCES: u64 = get_number_of_resources!();
pub const NUMBER_OF_COMPONENTS: u64 = get_number_of_components!();