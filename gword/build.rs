use gword_io::WordleMetadata;
//添加编译
fn main() {
    gear_wasm_builder::build_with_metadata::<WordleMetadata>();
}
