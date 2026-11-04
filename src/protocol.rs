crownos_ipc::protocol! {
    pub mod protocol: "crownlauncher" {
        /// Each returns whether the launcher is shown afterwards.
        fn show() -> bool;
        fn hide() -> bool;
        fn toggle() -> bool;
    }
}
