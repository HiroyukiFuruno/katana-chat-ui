use super::{NativeHost, metadata::HostMetadata};

#[test]
fn native_hosts_have_stable_window_titles() {
    assert_eq!(
        HostMetadata::for_host(NativeHost::Egui).window_title,
        "katana-chat-ui harness egui"
    );
    assert_eq!(
        HostMetadata::for_host(NativeHost::Floem).window_title,
        "katana-chat-ui harness floem"
    );
    assert_eq!(
        HostMetadata::for_host(NativeHost::Gpui).window_title,
        "katana-chat-ui harness gpui"
    );
}

#[test]
fn native_hosts_write_separate_output_files() {
    assert_eq!(
        HostMetadata::for_host(NativeHost::Egui).output_name,
        "native-egui.png"
    );
    assert_eq!(
        HostMetadata::for_host(NativeHost::Floem).output_name,
        "native-floem.png"
    );
    assert_eq!(
        HostMetadata::for_host(NativeHost::Gpui).output_name,
        "native-gpui.png"
    );
}
