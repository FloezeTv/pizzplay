use axum::{
    http::{header, HeaderValue},
    response::IntoResponse,
    routing::get,
    Router,
};
use include_dir::{include_dir, Dir, DirEntry, File};

static CLIENT_DIR: Dir<'_> = include_dir!("$CLIENT_BUILD_DIR");

/// Adds a file to a router.
/// If `filename_override` is `Some`, does not use the path of the 'file`,
/// but uses the override instead.
fn add_file_to_router(
    router: Router,
    file: &'static File<'_>,
    filename_override: Option<String>,
) -> Router {
    let mime = infer::get(file.contents())
        .map(|t| t.mime_type().to_owned())
        .unwrap_or_else(|| {
            mime_guess::from_path(file.path())
                .first_or_text_plain()
                .to_string()
        });
    let content = file.contents();

    router.route(
        filename_override
            .unwrap_or_else(|| {
                format!(
                    "/{}",
                    file.path().to_str().unwrap_or_else(|| panic!(
                        "Failed to add static client file: {:?}",
                        file.path()
                    ))
                )
            })
            .as_str(),
        get(move || async {
            let mime = mime;
            let mut response = content.into_response();
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime.as_ref()).unwrap(),
            );
            response
        }),
    )
}

/// Add a directory recursively to a router.
/// If `root_alias` is `Some`, aliases the directory ("/name" and "/name/") to the provided alias.
fn add_dir_to_router(router: Router, dir: &'static Dir<'_>, root_alias: Option<&str>) -> Router {
    dir.entries().into_iter().fold(router, |router, e| match e {
        DirEntry::Dir(d) => add_dir_to_router(router, d, root_alias),
        DirEntry::File(f) => {
            let mut router = add_file_to_router(router, f, None);
            if let Some(root_alias) = root_alias {
                if root_alias
                    == f.path()
                        .file_name()
                        .expect("File has no name")
                        .to_str()
                        .expect("Failed to get file name")
                {
                    let parent = f
                        .path()
                        .parent()
                        .map(|f| f.to_str().expect("Failed to get directory name"))
                        .unwrap_or("/");
                    if !parent.is_empty() {
                        router = add_file_to_router(router, f, Some(parent.to_owned()));
                    }
                    router = add_file_to_router(router, f, Some(parent.to_owned() + "/"));
                }
            }
            router
        }
    })
}

/// Creates a router containing all the files from the client directory,
/// as specified during build-time in the `CLIENT_BUILD_DIR` environment variable.
///
/// ## Parameters
/// - `root_alias`: An optional alias when accessing the directories directly
///   - `Some("index.html")`: Will serve `/foo/index.html` at `/foo` and `/foo/` as well.
///     Note that the root ("/") does not act without trailing slash when nesting due to how [axum] nests [Router]s
///   - `None`: Do not alias directories
pub fn client_handler(directory_alias: Option<&str>) -> Router {
    add_dir_to_router(Router::new(), &CLIENT_DIR, directory_alias)
}
