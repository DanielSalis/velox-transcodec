fn main() {
    build_qt_resize();
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent-dark".into())
        .embed_resources(slint_build::EmbedResourcesKind::EmbedFiles);
    slint_build::compile_with_config("ui/desktop.slint", config)
        .expect("compilar interface do Velox");
}

fn build_qt_resize() {
    println!("cargo:rustc-check-cfg=cfg(velox_qt_resize)");
    println!("cargo:rerun-if-changed=src/desktop/window_resize.cpp");
    println!("cargo:rerun-if-env-changed=SLINT_NO_QT");
    if std::env::var_os("SLINT_NO_QT").is_some()
        || std::env::var("DEP_QT_FOUND").as_deref() != Ok("1")
        || !std::env::var("DEP_QT_VERSION")
            .is_ok_and(|version| version.starts_with("6.") || version.starts_with("5.15"))
    {
        return;
    }

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .define("QT_NO_OPENGL", None)
        .include(std::env::var_os("DEP_QT_INCLUDE_PATH").expect("cabeçalhos Qt"))
        .file("src/desktop/window_resize.cpp");
    for flag in std::env::var("DEP_QT_COMPILE_FLAGS")
        .expect("flags do Qt")
        .split_terminator(';')
    {
        build.flag(flag);
    }
    build.compile("velox_qt_resize");
    println!("cargo:rustc-cfg=velox_qt_resize");
}
