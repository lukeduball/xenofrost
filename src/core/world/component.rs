pub trait Component {
    const ID: u64;

    fn get_component_id(&self) -> u64;
    fn component_id() -> u64;
}