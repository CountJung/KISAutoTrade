//! macOS-only public WK APIs. No existing configuration or data store is opened.
use crate::{generated, HostEvent};
use block2::RcBlock;
use objc2::{rc::Retained, AllocAnyThread, MainThreadMarker};
use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSImage};
use objc2_foundation::{NSDictionary, NSError, NSString, NSURL};
use objc2_web_kit::{WKContentRuleList, WKContentRuleListStore};
use tao::event_loop::EventLoopProxy;
use wry::{WebView, WebViewExtMacOS};

pub fn prepare_rules(
    view: &WebView,
    proxy: EventLoopProxy<HostEvent>,
) -> Result<Retained<WKContentRuleListStore>, String> {
    let mtm = MainThreadMarker::new().ok_or("WK setup must run on the main thread")?;
    // Wry selected this new store through with_incognito(true). Passing an
    // existing WK configuration would bypass that selection, so none is used.
    let persistent = unsafe {
        view.webview()
            .configuration()
            .websiteDataStore()
            .isPersistent()
    };
    if persistent {
        return Err("WK website data store is persistent; refusing asset navigation".into());
    }
    let store_path = NSString::from_str(&format!("{}/rules", generated::ROOT));
    let url = NSURL::fileURLWithPath_isDirectory(&store_path, true);
    let store = unsafe { WKContentRuleListStore::storeWithURL(Some(&url), mtm) }
        .ok_or("Could not create the private temporary content-rule store")?;
    let controller = view.manager();
    let callback = RcBlock::new(move |rule: *mut WKContentRuleList, error: *mut NSError| {
        if MainThreadMarker::new().is_none() || !error.is_null() || rule.is_null() {
            let _ = proxy.send_event(HostEvent::RulesFailed);
            return;
        }
        // WK invokes this callback with a live rule on the main thread. Install
        // it before the host permits any URL to load. No native message handler.
        unsafe { controller.addContentRuleList(&*rule) };
        let _ = proxy.send_event(HostEvent::RulesReady);
    });
    let identifier = NSString::from_str(generated::IDENTITY);
    let rules = NSString::from_str(
        r#"[{"trigger":{"url-filter":"^https?://"},"action":{"type":"block"}},{"trigger":{"url-filter":"^wss?://"},"action":{"type":"block"}},{"trigger":{"url-filter":"^file:"},"action":{"type":"block"}}]"#,
    );
    unsafe {
        store.compileContentRuleListForIdentifier_encodedContentRuleList_completionHandler(
            Some(&identifier),
            Some(&rules),
            Some(&callback),
        );
    }
    Ok(store)
}

/// Capture only this WKWebView, never the display or another application's window.
pub fn snapshot(view: &WebView, sequence: usize, proxy: EventLoopProxy<HostEvent>) {
    let output = format!("{}/output/snapshot-{sequence:02}.png", generated::ROOT);
    let callback = RcBlock::new(move |image: *mut NSImage, error: *mut NSError| {
        let success = (|| {
            if !error.is_null() || image.is_null() || MainThreadMarker::new().is_none() {
                return None;
            }
            // WebKit owns the callback image for this invocation.
            let tiff = unsafe { (&*image).TIFFRepresentation() }?;
            let bitmap = NSBitmapImageRep::initWithData(NSBitmapImageRep::alloc(), &tiff)?;
            let properties = NSDictionary::new();
            let png = unsafe {
                bitmap.representationUsingType_properties(NSBitmapImageFileType::PNG, &properties)
            }?;
            if png.length() > 20 * 1024 * 1024 {
                return None;
            }
            Some(png.writeToFile_atomically(&NSString::from_str(&output), true))
        })()
        .unwrap_or(false);
        let _ = proxy.send_event(HostEvent::SnapshotDone(sequence, success));
    });
    unsafe {
        view.webview()
            .takeSnapshotWithConfiguration_completionHandler(None, &callback);
    }
}
