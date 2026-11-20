use std::env;
use std::path::PathBuf;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let manifest_dir =
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));

        let icon_path = manifest_dir
            .join("../../assets/logo.ico")
            .canonicalize()
            .expect("failed to locate logo.ico");

        let mut resource = winresource::WindowsResource::new();

        resource.set_icon(icon_path.to_str().expect("invalid icon path"));

        resource
            .compile()
            .expect("failed to compile Windows resources");
    }
}
