//! Windows-only resource boundary for the isolated spike. Cancelling a
//! navigation is not a guarantee that WebView2 has sent no HTTP request.
//! Intercept all resource contexts and return a local empty denial instead.
use super::{GuardCounters, local_resource_origin};
use std::sync::{Arc, atomic::Ordering};
use tauri::{Manager, Url, WebviewWindow};
use webview2_com::{
    Microsoft::Web::WebView2::Win32::*, WebResourceRequestedEventHandler, take_pwstr,
};
use windows::core::{HSTRING, Interface, PWSTR};

pub(super) fn install(window: &WebviewWindow, guards: Arc<GuardCounters>) -> tauri::Result<()> {
    let app = window.app_handle().clone();
    window.with_webview(move |platform| {
        // SAFETY: Tauri invokes this closure on the owning WebView thread.
        // Controllers/environments are valid owned COM interfaces supplied by
        // Tauri. The callback owns its environment clone; URI buffers are freed
        // by the same CoTaskMem helper used by pinned Wry. No borrowed pointer
        // or event argument escapes a callback, and no payload is logged.
        let result: windows::core::Result<()> = unsafe {
            (|| {
                let webview = platform.controller().CoreWebView2()?;
                let environment = platform.environment();
                let filter = HSTRING::from("*");
                if let Ok(all_sources) = webview.cast::<ICoreWebView2_22>() {
                    all_sources.AddWebResourceRequestedFilterWithRequestSourceKinds(
                        &filter,
                        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                        COREWEBVIEW2_WEB_RESOURCE_REQUEST_SOURCE_KINDS_ALL,
                    )?;
                } else {
                    webview.AddWebResourceRequestedFilter(
                        &filter,
                        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                    )?;
                }
                let callback_guards = guards.clone();
                let callback_app = app.clone();
                let mut token = 0;
                webview.add_WebResourceRequested(
                    &WebResourceRequestedEventHandler::create(Box::new(move |_, args| {
                        let Some(args) = args else {
                            callback_guards
                                .resource_guard_failed
                                .store(true, Ordering::Relaxed);
                            callback_app.exit(1);
                            return Ok(());
                        };
                        let allowed = (|| -> windows::core::Result<bool> {
                            let request = args.Request()?;
                            let mut uri = PWSTR::null();
                            request.Uri(&mut uri)?;
                            let uri = take_pwstr(uri);
                            Ok(Url::parse(&uri).is_ok_and(|url| local_resource_origin(&url)))
                        })()
                        .unwrap_or(false);
                        if !allowed {
                            let denied = (|| -> windows::core::Result<()> {
                                let response = environment.CreateWebResourceResponse(
                                    None,
                                    403,
                                    &HSTRING::from("Blocked"),
                                    &HSTRING::from(
                                        "Content-Length: 0\r\nCache-Control: no-store\r\n",
                                    ),
                                )?;
                                args.SetResponse(&response)
                            })();
                            if let Err(ref error) = denied {
                                callback_guards
                                    .resource_guard_failed
                                    .store(true, Ordering::Relaxed);
                                println!(
                                    "LINTRA_RUNTIME_REQUEST_GUARD_FAILURE {:#x}",
                                    error.code().0
                                );
                                callback_app.exit(1);
                            }
                            denied?;
                            callback_guards
                                .web_resource_denied
                                .fetch_add(1, Ordering::Relaxed);
                            println!("LINTRA_RUNTIME_WEB_RESOURCE_DENIED");
                        }
                        Ok(())
                    })),
                    &mut token,
                )?;
                Ok(())
            })()
        };
        match result {
            Ok(()) => guards.resource_guard_ready.store(true, Ordering::Relaxed),
            Err(error) => {
                guards.resource_guard_failed.store(true, Ordering::Relaxed);
                println!("LINTRA_RUNTIME_REQUEST_GUARD_FAILURE {:#x}", error.code().0);
                app.exit(1);
            }
        }
    })
}
