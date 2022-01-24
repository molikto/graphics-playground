#[macro_export]
macro_rules! ifelse {
  ($c:expr, $v:expr, $v1:expr) => {
      if $c {$v} else {$v1}
  };
}


#[macro_export]
macro_rules! unwrap_or_return {
  ( $e:expr ) => {
      match $e {
          Some(x) => x,
          None => return
      }
  };
}

#[macro_export]
macro_rules! unwrap_or_return_val {
  ( $e:expr, $v:expr ) => {
      match $e {
          Some(x) => x,
          None => return $v
      }
  };
}


// macro_rules! true_or_return {
//   ( $e:expr ) => {
//       if !($e) {
//           return;
//       }
//   };
// }


#[macro_export]
macro_rules! if_true_return {
  ($expr:expr) => {
      if $expr {
        return;
      }
  };
}

pub unsafe fn very_bad_function<T>(reference: &T) -> &mut T {
  let const_ptr = reference as *const T;
  let mut_ptr = const_ptr as *mut T;
  &mut *mut_ptr
}

pub trait EndianSwap {
    fn swap_endian(self) -> Self;
}

impl EndianSwap for u32 {
    fn swap_endian(self) -> Self {
      let x = self;
      return ((x & 0xFF) << 24) | ((x & 0xFF00) << 8) | ((x & 0xFF0000) >> 8) | ((x & 0xFF000000) >> 24);
    }
  }

#[cfg(not(target_arch = "spirv"))]
pub use super::lang_std::*;