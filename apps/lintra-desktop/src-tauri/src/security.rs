use tauri::Url;

pub(crate) fn local_asset_origin(url: &Url) -> bool {
    url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
        && matches!(
            (url.scheme(), url.host_str()),
            ("tauri", Some("localhost")) | ("http", Some("tauri.localhost"))
        )
}

#[cfg(any(target_os = "windows", test))]
pub(crate) fn local_resource_origin(url: &Url) -> bool {
    local_asset_origin(url)
        || (url.username().is_empty()
            && url.password().is_none()
            && url.port().is_none()
            && matches!(
                (url.scheme(), url.host_str()),
                ("http", Some("ipc.localhost")) | ("ipc", Some("localhost"))
            ))
}

#[cfg(any(target_os = "windows", test))]
pub(crate) fn restrict_document_connections(headers: &mut tauri::http::HeaderMap) {
    headers.insert(
        "connection-allowlist",
        tauri::http::HeaderValue::from_static(
            r#"("http://tauri.localhost/*" "http://ipc.localhost/*")"#,
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_exact_asset_and_ipc_origins_are_allowed() {
        for value in ["tauri://localhost/index.html", "http://tauri.localhost/"] {
            let url = Url::parse(value).unwrap();
            assert!(local_asset_origin(&url));
            assert!(local_resource_origin(&url));
        }
        for value in ["ipc://localhost/", "http://ipc.localhost/"] {
            let url = Url::parse(value).unwrap();
            assert!(!local_asset_origin(&url));
            assert!(local_resource_origin(&url));
        }
        for value in [
            "https://tauri.localhost/",
            "http://tauri.localhost.evil/",
            "http://localhost/",
            "http://127.0.0.1/",
            "file:///tmp/record",
            "http://tauri.localhost:9000/",
            "tauri://user@localhost/",
            "http://user:pass@ipc.localhost/",
            "ipc://localhost:9000/",
        ] {
            let url = Url::parse(value).unwrap();
            assert!(!local_asset_origin(&url), "{value}");
            assert!(!local_resource_origin(&url), "{value}");
        }
    }

    #[test]
    fn connection_header_keeps_only_internal_endpoints() {
        let mut headers = tauri::http::HeaderMap::new();
        restrict_document_connections(&mut headers);
        assert_eq!(
            headers["connection-allowlist"],
            r#"("http://tauri.localhost/*" "http://ipc.localhost/*")"#
        );
    }
}
