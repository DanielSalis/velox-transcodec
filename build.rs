fn main() {
    build_qt_resize();
    build_windows_icon();
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent-dark".into())
        .embed_resources(slint_build::EmbedResourcesKind::EmbedFiles);
    slint_build::compile_with_config("ui/desktop.slint", config)
        .expect("compilar interface do Velox");
}

fn build_windows_icon() {
    println!("cargo:rerun-if-changed=packaging/icons/windows/Velox.ico");
    println!("cargo:rerun-if-changed=packaging/icons/windows/velox.rc");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    assert_eq!(
        std::env::var("CARGO_CFG_TARGET_ENV").as_deref(),
        Ok("msvc"),
        "O recurso de ícone Windows requer o toolchain MSVC"
    );

    let target = std::env::var("TARGET").expect("target Cargo");
    let mut rc = cc::windows_registry::find_tool(&target, "rc.exe")
        .map(|tool| tool.to_command())
        .unwrap_or_else(|| std::process::Command::new("rc.exe"));
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR"))
        .join("velox-icon.res");
    let status = rc
        .current_dir("packaging/icons/windows")
        .arg("/nologo")
        .arg(format!("/fo{}", output.display()))
        .arg("velox.rc")
        .status()
        .expect("executar rc.exe do Windows SDK");
    assert!(
        status.success(),
        "não foi possível compilar o ícone Windows"
    );
    println!(
        "cargo:rustc-link-arg-bin=velox-transcode={}",
        output.display()
    );
}

fn build_qt_resize() {
    println!("cargo:rustc-check-cfg=cfg(velox_qt_resize)");
    println!("cargo:rerun-if-changed=src/desktop/window_resize.cpp");
    println!("cargo:rerun-if-changed=src/desktop/macos.cpp");
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
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let qt_libraries = std::env::var("DEP_QT_LIBRARY_PATH").expect("bibliotecas Qt");
        println!("cargo:rustc-link-arg-bin=velox-transcode=-Wl,-rpath,{qt_libraries}");
        build.file("src/desktop/macos.cpp");
    }
    build.compile("velox_qt_resize");
    println!("cargo:rustc-cfg=velox_qt_resize");
}
