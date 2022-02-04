



pub unsafe fn extend_lifetime<'b, V>(r: &'b V) -> &'static V {
    std::mem::transmute::<&'b V, &'static V>(r)
}

#[macro_export]
macro_rules! cargo_manifest_dir {
    () => {
        std::path::Path::new(std::env!("CARGO_MANIFEST_DIR")).to_path_buf()
    };
}