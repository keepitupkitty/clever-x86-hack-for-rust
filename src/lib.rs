#![feature(c_variadic, phantom_variance_markers, core_intrinsics)]

use {
  core::{
    ffi::{VaList, c_char},
    fmt,
    marker::PhantomCovariantLifetime
  },
  std::intrinsics::{va_copy, va_end}
};

// X86-64
#[cfg(target_arch = "x86_64")]
mod this {

  use core::ffi::c_void;

  #[repr(C)]
  #[derive(Debug, Clone, Copy)]
  pub struct ExtVaListInner {
    pub gp_offset: i32,
    pub fp_offset: i32,
    pub overflow_arg_area: *const c_void,
    pub reg_save_area: *const c_void
  }
}

#[repr(transparent)]
pub struct ExtVaList<'a> {
  inner: this::ExtVaListInner,
  _marker: PhantomCovariantLifetime<'a>
}

impl<'a> ExtVaList<'a> {
  #[inline]
  pub unsafe fn from_va_list(va: VaList<'a>) -> Self {
    let orig = core::mem::size_of::<VaList>();
    let ext = core::mem::size_of::<Self>();

    assert_eq!(orig, ext, "Sizes between VaList and ExtVaList differ");

    let align_orig = core::mem::align_of::<VaList>();
    let align_ext = core::mem::align_of::<Self>();

    assert_eq!(
      align_orig, align_ext,
      "Alignments between VaList and ExtVaList differ"
    );

    unsafe { core::mem::transmute(va) }
  }

  #[inline]
  pub unsafe fn into_va_list(self) -> VaList<'a> {
    unsafe { core::mem::transmute(self) }
  }
}

impl<'f> ExtVaList<'f> {
  #[inline]
  #[cfg(target_arch = "x86_64")]
  pub unsafe fn get_long_double_bits(&mut self) -> [u8; 16] {
    let src = self.inner.overflow_arg_area as *const [u8; 16];
    let result = unsafe { src.read() };

    unsafe {
      self.inner.overflow_arg_area = self.inner.overflow_arg_area.add(16)
    };

    result
  }
}

impl fmt::Debug for ExtVaList<'_> {
  fn fmt(
    &self,
    f: &mut fmt::Formatter<'_>
  ) -> fmt::Result {
    f.debug_tuple("ExtVaList").field(&self.inner).finish()
  }
}

impl Clone for ExtVaList<'_> {
  #[inline]
  fn clone(&self) -> Self {
    unsafe { core::mem::transmute(va_copy(core::mem::transmute(self))) }
  }
}

impl<'f> Drop for ExtVaList<'f> {
  fn drop(&mut self) {
    unsafe { va_end(core::mem::transmute(self)) }
  }
}

// TEST CODE
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_test_c(
  s: *const c_char,
  mut args: ...
) {
  let mut t = unsafe { ExtVaList::from_va_list(args.clone()) };

  let s = unsafe { core::ffi::CStr::from_ptr(s).to_str().unwrap() };

  println!("VaList start: {:#?}", args);
  println!("ExtVaList start: {:#?}\n", t);

  let ld = unsafe { t.get_long_double_bits() };

  println!("Your string: {s}");
  println!("Your long double: {:#?}", ld);

  args = unsafe { t.clone().into_va_list() };

  unsafe { println!("Potential size_t: {}", args.arg::<usize>()) };

  let mut t2 =
    unsafe { ExtVaList::from_va_list(args.clone()) };
  let ld2 = unsafe { t2.get_long_double_bits() };

  println!("Your long double 2: {:#?}\n", ld2);

  println!("VaList end: {:#?}", args);
  println!("ExtVaList end: {:#?}", t);
  println!("ExtVaList2: {:#?}", t2);
}

#[test]
fn cmp_sizes() {
  let orig = core::mem::size_of::<VaList>();
  let ext = core::mem::size_of::<ExtVaList>();

  println!("VaList size: {}", orig);
  println!("ExtVaList size: {}", ext);

  assert_eq!(orig, ext, "Sizes between VaList and ExtVaList differ");

  let align_orig = core::mem::align_of::<VaList>();
  let align_ext = core::mem::align_of::<ExtVaList>();

  assert_eq!(
    align_orig, align_ext,
    "Alignments between VaList and ExtVaList differ"
  );
}
