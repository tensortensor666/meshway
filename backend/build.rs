fn main() {
    #[cfg(windows)]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("../packaging/windows/meshway.ico");
        resource
            .compile()
            .expect("failed to compile Windows resources");
    }
}
