// E2e tests for CdpClient::attach_android (adb-based Android WebView remote
// debugging). Requires the `android` feature:
//   cargo test --features android --test adb_locator_e2e_test
//
// Tests that need a real connected Android device with `adb` installed and a
// debuggable WebView active are marked #[ignore] with
// "requires a real Android device" and were NOT run in the environment these
// were written in (no `adb` binary and no device/emulator available there) —
// verify the happy path on real hardware before relying on it.
#![cfg(feature = "android")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use chromiumctl::CdpClient;

/// @covers: attach_android
#[test]
fn test_attach_android_fails_with_actionable_error_when_adb_path_invalid() {
    // Safety: no other test in this binary reads or writes ADB_PATH.
    unsafe {
        std::env::set_var("ADB_PATH", "/nonexistent/adb");
    }
    let result = CdpClient::attach_android("com.example.app");
    unsafe {
        std::env::remove_var("ADB_PATH");
    }

    let err = result.err().expect("attach_android must fail when ADB_PATH is unreachable");
    assert!(
        err.contains("ADB_PATH"),
        "error should name the offending env var so the caller can fix it: {err}"
    );
}

/// @covers: attach_android
#[test]
#[ignore = "requires a real Android device/emulator with adb installed and a debuggable WebView active"]
fn test_attach_android_connects_to_real_webview() {
    // Replace with a real debuggable package name on the connected device.
    let client = CdpClient::attach_android("com.example.app")
        .expect("attach_android must succeed against a real debuggable WebView");
    assert!(client.port() > 0);
}

/// @covers: attach_android
#[test]
#[ignore = "requires a real Android device/emulator with adb installed"]
fn test_attach_android_fails_when_package_not_debuggable() {
    let result = CdpClient::attach_android("com.definitely.not.a.debuggable.package");
    assert!(
        result.is_err(),
        "attach_android must fail cleanly for a package with no active WebView debug socket"
    );
}
