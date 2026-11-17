//! Offline research entry. Deliberately independent of the operational app setup.
//!
//! No AppState, profile/config loader, storage, plugins, HTTP server or daemons are
//! installed. Even if a user navigates this window to another bundled page, its
//! IPC handler has no account, network-provider or trading commands.

pub const IDENTIFIER: &str = "com.countjung.kisautotrade.research";
pub const RESEARCH_MARKER: &str =
    "Object.defineProperty(window, '__KIS_RESEARCH_ONLY__', { value: true, writable: false, configurable: false });";

pub fn run() {
    let mut context = crate::app_context();
    // Also isolate feature-only cargo runs where the CLI config was omitted.
    // Packaging uses tauri.research.conf.json so the .app identifier matches.
    context.config_mut().identifier = IDENTIFIER.into();
    context.config_mut().app.windows.clear();
    tauri::Builder::default()
        .setup(|app| {
            tauri::WebviewWindowBuilder::new(
                app,
                "sector-research",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("KISAutoTrade Research — 로컬 자료 전용")
            .inner_size(1200.0, 850.0)
            .min_inner_size(375.0, 600.0)
            .incognito(true)
            .initialization_script(RESEARCH_MARKER)
            .build()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::commands::preview_sector_portfolio,
            crate::commands::preview_broker_chart_portfolio,
            crate::commands::validate_sector_data,
        ])
        .run(context)
        .expect("연구 전용 앱을 시작하지 못했습니다");
}

#[cfg(test)]
mod tests {
    #[test]
    fn research_package_has_separate_identity_and_no_operational_windows() {
        let research: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.research.conf.json")).unwrap();
        let normal: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(research["identifier"], super::IDENTIFIER);
        assert_ne!(research["identifier"], normal["identifier"]);
        assert_eq!(research["app"]["windows"], serde_json::json!([]));
    }
}
