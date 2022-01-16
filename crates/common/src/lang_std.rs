


pub unsafe fn extend_lifetime<'b, V>(r: &'b V) -> &'static V {
    std::mem::transmute::<&'b V, &'static V>(r)
}