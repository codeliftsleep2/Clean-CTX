use std::path::Path;

/// Move a prepared, not-yet-opened source fixture below its owning temporary
/// directory. The owner still cleans up source and sibling runtime files.
#[cfg(all(feature = "typescript", feature = "angular"))]
pub(crate) fn isolate_program(root: &Path) -> std::path::PathBuf {
    let project = root.join("project");
    assert!(root.is_absolute() && project.starts_with(root));
    assert!(!project.exists());
    let entries: Vec<_> = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    std::fs::create_dir(&project).unwrap();
    for source in entries {
        let target = project.join(source.file_name().unwrap());
        assert!(source.starts_with(root) && target.starts_with(root));
        std::fs::rename(source, target).unwrap();
    }
    project
}

/// Own source and runtime storage together without making SQLite sidecars
/// members of the TypeScript program's witnessed directory inventory.
#[cfg(all(feature = "typescript", feature = "angular"))]
pub(crate) struct PersistentProject {
    owner: tempfile::TempDir,
    project: std::path::PathBuf,
}

#[cfg(all(feature = "typescript", feature = "angular"))]
impl PersistentProject {
    pub(crate) fn new() -> Self {
        let owner = tempfile::tempdir().unwrap();
        let project = owner.path().join("project");
        std::fs::create_dir(&project).unwrap();
        Self { owner, project }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.project
    }

    pub(crate) fn database(&self, name: &str) -> std::path::PathBuf {
        self.owner.path().join(name)
    }
}

/// Supply canonical package evidence for hydration fixtures that use Angular APIs.
/// Package inputs stay under node_modules, outside source discovery candidates.
pub(crate) fn install(root: &Path) {
    let package = root.join("node_modules/@angular/core");
    std::fs::create_dir_all(&package).expect("Angular fixture package");
    let files = [
        (
            package.join("package.json"),
            r#"{"name":"@angular/core","version":"20.0.0","types":"index.d.ts"}"#,
        ),
        (
            package.join("index.d.ts"),
            "export declare function Injectable(metadata?: unknown): any;\nexport declare function Component(metadata: unknown): any;\n",
        ),
        (
            root.join("tsconfig.json"),
            r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#,
        ),
    ];
    for (path, contents) in files {
        if path.exists() {
            assert_eq!(
                std::fs::read_to_string(&path).expect("existing fixture input"),
                contents,
                "conflicting Angular fixture input: {}",
                path.display()
            );
        } else {
            std::fs::write(path, contents).expect("Angular fixture input");
        }
    }
}

/// Close the package graph used by the public NgRx dispatch preservation cases.
#[cfg(all(feature = "typescript", feature = "angular"))]
pub(crate) fn install_store_apis(root: &Path) {
    install(root);
    let package = root.join("node_modules/@ngrx/store");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@ngrx/store","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("index.d.ts"),
        "export interface Action { readonly type: string; }\nexport declare class Store<T> { dispatch(action: Action): void; }\n",
    ).unwrap();
}

/// Canonical package inputs for the existing DI preservation fixtures.
#[cfg(all(feature = "typescript", feature = "angular"))]
pub(crate) fn install_inject_apis(root: &Path) {
    install(root);
    std::fs::write(
        root.join("node_modules/@angular/core/index.d.ts"),
        "export declare function Injectable(metadata?: unknown): any;\nexport declare function Component(metadata: unknown): any;\nexport declare function Directive(metadata: unknown): any;\nexport declare function Pipe(metadata: unknown): any;\nexport declare function inject(token: unknown, options?: unknown): unknown;\n",
    )
    .expect("DI fixture package declarations");
}

/// Canonical package inputs for module contributor lifecycle fixtures.
#[cfg(all(feature = "typescript", feature = "angular"))]
pub(crate) fn install_module_apis(root: &Path) {
    install(root);
    std::fs::write(
        root.join("node_modules/@angular/core/index.d.ts"),
        "export declare function Injectable(metadata?: unknown): any;\nexport declare function Component(metadata: unknown): any;\nexport declare function Directive(metadata: unknown): any;\nexport declare function Pipe(metadata: unknown): any;\nexport declare function NgModule(metadata: unknown): any;\n",
    )
    .expect("module fixture package declarations");
}
