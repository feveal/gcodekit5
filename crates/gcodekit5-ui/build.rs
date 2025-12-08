use chrono::Utc;

fn main() {
    // Generate build timestamp
    let build_date = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);

    glib_build_tools::compile_resources(
        &["resources"],
        "resources/gresources.xml",
        "gcodekit5.gresource",
    );
}
