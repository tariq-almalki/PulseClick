fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/pulseclick.ico");
        resource
            .compile()
            .expect("failed to embed the PulseClick Windows icon");
    }
}
