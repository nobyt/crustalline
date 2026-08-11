use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use crustalline_ipc_types::RenderRequest;
use tauri::{Listener, WebviewUrl, WebviewWindowBuilder};

use crate::cli::RenderArgs;
use crate::gui::base_builder;

const RENDER_TIMEOUT: Duration = Duration::from_secs(15);

/// Hidden-window render pipeline (plan §5): loads the same frontend bundle as
/// the GUI in an invisible window, waits for it to render a 3Dmol.js frame
/// and hand back a PNG data URL, then writes the PNG to disk.
pub fn run_render(args: RenderArgs) {
    let req = RenderRequest {
        smiles: args.smiles,
        width: args.width,
        height: args.height,
        out_path: args.out_path.clone(),
    };
    let out_path = args.out_path;

    let done = Arc::new(AtomicBool::new(false));
    let done_for_listener = done.clone();

    let app = base_builder()
        .setup(move |app| {
            let init_script = format!(
                "window.__CRUSTALLINE_HEADLESS__ = true; window.__CRUSTALLINE_RENDER_REQUEST__ = {};",
                serde_json::to_string(&req).expect("RenderRequest is always serializable")
            );

            let app_handle = app.handle().clone();
            let out_path = out_path.clone();
            app.once("frame-ready", move |event| {
                match write_png_from_data_url(event.payload(), &out_path) {
                    Ok(()) => {
                        done_for_listener.store(true, Ordering::SeqCst);
                        app_handle.exit(0);
                    }
                    Err(e) => {
                        eprintln!("crustalline render: {e}");
                        app_handle.exit(1);
                    }
                }
            });

            WebviewWindowBuilder::new(app, "headless", WebviewUrl::App("index.html".into()))
                .visible(false)
                .inner_size(req.width as f64, req.height as f64)
                .initialization_script(&init_script)
                .build()?;

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build crustalline app for headless render");

    let app_handle_timeout = app.handle().clone();
    std::thread::spawn(move || {
        std::thread::sleep(RENDER_TIMEOUT);
        if !done.load(Ordering::SeqCst) {
            eprintln!(
                "crustalline render: timed out after {}s waiting for the webview to render a \
                 frame. If this is a headless/remote Linux host, see docs/headless-rendering.md \
                 — you likely need to run under Xvfb (xvfb-run -a crustalline render ...).",
                RENDER_TIMEOUT.as_secs()
            );
            app_handle_timeout.exit(1);
        }
    });

    app.run(|_, _| {});
}

fn write_png_from_data_url(payload: &str, out_path: &str) -> Result<(), String> {
    let data_url: String =
        serde_json::from_str(payload).map_err(|e| format!("bad frame-ready payload: {e}"))?;
    let b64 = data_url.strip_prefix("data:image/png;base64,").ok_or_else(|| {
        format!(
            "unexpected data URL prefix (want data:image/png;base64,...): {:.40}",
            data_url
        )
    })?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| format!("failed to decode PNG data URL: {e}"))?;
    std::fs::write(out_path, &bytes).map_err(|e| format!("failed to write {out_path}: {e}"))?;
    eprintln!("crustalline render: wrote {out_path} ({} bytes)", bytes.len());
    Ok(())
}
