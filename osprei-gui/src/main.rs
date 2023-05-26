use leptos::mount_to_body;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    mount_to_body(osprei_gui::osprei_gui)
}
