//! Independent UI renderer. Never link/import the production application crate.
mod generated;
mod native;

use serde_json::{json, Value};
use std::{
    borrow::Cow,
    fs,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::{
    http::{header, Request, Response},
    NewWindowResponse, PermissionResponse, WebViewBuilder,
};

#[derive(Debug)]
enum HostEvent {
    RulesReady,
    RulesFailed,
    Report(String),
    ReportTooLarge,
    SnapshotDone(usize, bool),
}

fn write_json(name: &str, value: &Value) {
    // Fixed names or a bounded native sequence; no filename comes from page content.
    let Ok(bytes) = serde_json::to_vec_pretty(value) else {
        return;
    };
    let bytes = if bytes.len() <= 256 * 1024 {
        bytes
    } else {
        br#"{"artifactTooLarge":true}"#.to_vec()
    };
    let _ = fs::write(format!("{}/output/{name}", generated::ROOT), bytes);
}

fn record(events: &Arc<Mutex<Vec<Value>>>, value: Value) {
    if let Ok(mut events) = events.lock() {
        if events.len() < 255 {
            events.push(value);
        } else if events.len() == 255 {
            events.push(json!({"nativeLedgerOverflow":true}));
        }
    }
}

fn permitted_navigation(url: &str) -> bool {
    let Ok(uri) = url.parse::<wry::http::Uri>() else {
        return false;
    };
    uri.scheme_str() == Some("kis-smoke")
        && uri
            .authority()
            .is_some_and(|authority| authority.as_str() == "localhost")
        && [
            "/",
            "/index.html",
            "/settings",
            "/strategy",
            "/trading",
            "/history",
            "/log",
        ]
        .contains(&uri.path())
        && uri.query().is_none()
}

fn resource(
    request: Request<Vec<u8>>,
    events: &Arc<Mutex<Vec<Value>>>,
) -> Response<Cow<'static, [u8]>> {
    let uri = request.uri();
    let allowed = request.method() == "GET"
        && uri.scheme_str() == Some("kis-smoke")
        && uri
            .authority()
            .is_some_and(|authority| authority.as_str() == "localhost")
        && uri.query().is_none();
    let path = if permitted_navigation(&uri.to_string()) {
        "/index.html"
    } else {
        uri.path()
    };
    let asset = allowed
        .then(|| generated::ASSETS.iter().find(|asset| asset.0 == path))
        .flatten();
    record(
        events,
        json!({"kind":"asset","path":uri.path().chars().take(256).collect::<String>(),"allowed":asset.is_some()}),
    );
    match asset {
        Some((_, bytes, mime)) => Response::builder()
            .header(header::CONTENT_TYPE, *mime)
            .header(header::CONTENT_SECURITY_POLICY, generated::CSP)
            .header(header::CACHE_CONTROL, "no-store")
            .header("X-Content-Type-Options", "nosniff")
            .body(Cow::Borrowed(*bytes))
            .expect("static asset response"),
        None => Response::builder()
            .status(403)
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_SECURITY_POLICY, "default-src 'none'")
            .body(Cow::Borrowed(
                b"Synthetic smoke denied this resource".as_slice(),
            ))
            .expect("static denial"),
    }
}

// Only rendered state participates. Request counts, errors, timestamps and
// focus churn must not consume the bounded image budget.
fn screen_state(report: &Value) -> Value {
    let canvas = report
        .get("canvas")
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().map(|item| json!({
            "width":item.get("width"), "height":item.get("height"),
            "cssWidth":item.get("cssWidth").and_then(Value::as_f64).map(f64::round),
            "cssHeight":item.get("cssHeight").and_then(Value::as_f64).map(f64::round),
            "painted":item.get("sampledColors").and_then(Value::as_u64).unwrap_or(0) > 3,
        })).collect::<Vec<_>>()
        })
        .unwrap_or_default();
    json!({"route":report.get("location"),"dialogCount":report.get("dialogCount"),
        "bodyBg":report.get("bodyBg"),"viewport":report.get("viewport"),"canvas":canvas})
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Output/rules directories were created by the build-only driver under its
    // own fresh /tmp root. No home/config/keychain/profile resolution is used.
    let started = Instant::now();
    let event_loop = EventLoopBuilder::<HostEvent>::with_user_event().build();
    let window = WindowBuilder::new()
        .with_title(format!(
            "OFFLINE SYNTHETIC UI SMOKE 8 — {}",
            generated::IDENTITY
        ))
        .with_inner_size(LogicalSize::new(1280.0, 900.0))
        .with_max_inner_size(LogicalSize::new(1920.0, 1440.0))
        .build(&event_loop)?;
    let events = Arc::new(Mutex::new(Vec::<Value>::new()));
    let asset_events = events.clone();
    let navigation_events = events.clone();
    let popup_events = events.clone();
    let download_events = events.clone();
    let permission_events = events.clone();
    let bootstrap = include_str!("bootstrap.js")
        .replace("__SMOKE_FIXTURE_JSON__", include_str!("fixtures.json"));
    let webview = WebViewBuilder::new()
        .with_incognito(true)
        .with_devtools(false)
        .with_initialization_script(bootstrap)
        .with_custom_protocol("kis-smoke".into(), move |_, request| resource(request, &asset_events))
        .with_navigation_handler(move |url| {
            let allowed = permitted_navigation(&url);
            record(&navigation_events, json!({"kind":"navigation","url":url.chars().take(256).collect::<String>(),"allowed":allowed}));
            allowed
        })
        .with_new_window_req_handler(move |_, _| {
            record(&popup_events, json!({"kind":"popup","allowed":false}));
            NewWindowResponse::Deny
        })
        .with_download_started_handler(move |_, _| {
            record(&download_events, json!({"kind":"download","allowed":false}));
            false
        })
        .with_permission_handler(move |_| {
            record(&permission_events, json!({"kind":"permission","allowed":false}));
            PermissionResponse::Deny
        })
        // Intentionally no URL, HTML, existing WK configuration, or IPC handler.
        .build(&window)?;
    let _rule_store = match native::prepare_rules(&webview, event_loop.create_proxy()) {
        Ok(store) => store,
        Err(error) => {
            write_json(
                "host.json",
                &json!({"stage":"refused-before-assets","error":error}),
            );
            return Err(error.into());
        }
    };
    let mut ready = false;
    let mut report_pending = false;
    let mut last_report = Instant::now();
    let mut snapshots = 0usize;
    let mut snapshots_pending = false;
    let mut last_snapshot_state: Option<Value> = None;
    write_json(
        "host.json",
        &json!({"stage":"rules-pending","identity":generated::IDENTITY,"incognito":true,"persistent":false,"ipcRegistered":false}),
    );
    let host_proxy = event_loop.create_proxy();
    event_loop.run(move |event, _target, flow| {
        *flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(250));
        if started.elapsed() > Duration::from_secs(600) || (!ready && started.elapsed() > Duration::from_secs(15)) {
            write_json("host.json", &json!({"stage":"timeout","ready":ready}));
            *flow = ControlFlow::Exit;
            return;
        }
        match event {
            Event::UserEvent(HostEvent::RulesReady) => {
                ready = true;
                write_json("host.json", &json!({"stage":"rules-installed","identity":generated::IDENTITY,"persistent":false,"ipcRegistered":false,"maxRuntimeSeconds":600,"snapshotMode":"screen-state-change","observationIntervalSeconds":2,"maxSnapshots":12}));
                if let Err(error) = webview.load_url("kis-smoke://localhost/") {
                    write_json("host.json", &json!({"stage":"asset-load-error","error":error.to_string()}));
                    *flow = ControlFlow::Exit;
                }
            }
            Event::UserEvent(HostEvent::RulesFailed) => {
                write_json("host.json", &json!({"stage":"rules-failed-before-assets"}));
                *flow = ControlFlow::Exit;
            }
            Event::UserEvent(HostEvent::Report(serialized)) => {
                report_pending = false;
                let report = serde_json::from_str::<Value>(&serialized).unwrap_or(Value::Null);
                let report = match report {
                    Value::String(inner) => serde_json::from_str::<Value>(&inner).unwrap_or(Value::Null),
                    other => other,
                };
                let state = screen_state(&report);
                let mounted = report.get("rootChildren").and_then(Value::as_u64).unwrap_or(0) > 0;
                if mounted && snapshots < 12 && !snapshots_pending && last_snapshot_state.as_ref() != Some(&state) {
                    snapshots += 1;
                    snapshots_pending = true;
                    last_snapshot_state = Some(state);
                    native::snapshot(&webview, snapshots, host_proxy.clone());
                }
                let ledger = events.lock().map(|events| events.clone()).unwrap_or_default();
                write_json("report.json", &json!({"native":{"identity":generated::IDENTITY,"persistent":false,"rulesInstalled":ready,"ipcRegistered":false,"snapshotMode":"screen-state-change","snapshotCount":snapshots,"snapshotLimitReached":snapshots >= 12},"frontend":report,"nativeEvents":ledger}));
            }
            Event::UserEvent(HostEvent::ReportTooLarge) => {
                write_json("report.json", &json!({"failed":true,"reason":"frontendReportTooLarge","maxBytes":65536}));
                write_json("host.json", &json!({"stage":"failed","reason":"frontendReportTooLarge"}));
                *flow = ControlFlow::Exit;
                return;
            }
            Event::UserEvent(HostEvent::SnapshotDone(sequence, success)) => {
                snapshots_pending = false;
                let result = json!({"sequence":sequence,"success":success,"scope":"own WKWebView only","screenState":last_snapshot_state});
                write_json("snapshot.json", &result);
                write_json(&format!("snapshot-{sequence:02}.json"), &result);
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *flow = ControlFlow::Exit,
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                let _ = webview.set_bounds(wry::Rect { position: wry::dpi::PhysicalPosition::new(0,0).into(), size: wry::dpi::PhysicalSize::new(size.width, size.height).into() });
            }
            _ => {}
        }
        // Read-only observations; route changes and UI controls are performed
        // only by the human/root's explicit CUA actions, never by this host.
        if ready && !report_pending && last_report.elapsed() >= Duration::from_secs(2) {
            report_pending = true;
            last_report = Instant::now();
            let proxy = host_proxy.clone();
            let result = webview.evaluate_script_with_callback(
                "JSON.stringify(typeof window.__SMOKE_READ_REPORT__ === 'function' ? window.__SMOKE_READ_REPORT__() : {bootstrapMissing:true})",
                move |value| {
                    let event = if value.len() <= 65536 { HostEvent::Report(value) } else { HostEvent::ReportTooLarge };
                    let _ = proxy.send_event(event);
                },
            );
            if result.is_err() { report_pending = false; }
        }
    });
}
