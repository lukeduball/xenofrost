use xenofrost::core::engine::run;

fn main() {
    pollster::block_on(run());
}
