pub use common_no_std::lang::*;

pub unsafe fn extend_lifetime<'b, V>(r: &'b V) -> &'static V {
  std::mem::transmute::<&'b V, &'static V>(r)
}