
# compile (native, linux)
```bash
apt-get update
apt-get install gcc g++
cargo run --example basic
```

# run benchmark
`cargo bench`

## bench results
- spectral
    - rlua : 41ms
    - lua-rs : 105ms

# profiling (linux)
```bash
apt-get install valgrind
valgrind --tool=callgrind target/release/examples/spectral
callgrind_annotate callgrind.out.NNNNN
```

```
--------------------------------------------------------------------------------
Ir                     
--------------------------------------------------------------------------------
4,997,722,080 (100.0%)  PROGRAM TOTALS

--------------------------------------------------------------------------------
Ir                      file:function
--------------------------------------------------------------------------------
1,270,992,437 (25.43%)  src/vm.rs:lua_rs::vm::<impl lua_rs::state::LuaState>::vexecute [/home/jice/lua-rs/target/debug/examples/spectral]
  448,403,472 ( 8.97%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/slice/index.rs:<usize as core::slice::index::SliceIndex<[T]>>::index [/home/jice/lua-rs/target/debug/examples/spectral]
  373,669,955 ( 7.48%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index [/home/jice/lua-rs/target/debug/examples/spectral]
  245,725,530 ( 4.92%)  src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone [/home/jice/lua-rs/target/debug/examples/spectral]
  237,529,204 ( 4.75%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> [/home/jice/lua-rs/target/debug/examples/spectral]
  199,290,712 ( 3.99%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/metadata.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index
  123,032,646 ( 2.46%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/slice/index.rs:<usize as core::slice::index::SliceIndex<[T]>>::index_mut [/home/jice/lua-rs/target/debug/examples/spectral]
  118,071,086 ( 2.36%)  src/object.rs:lua_rs::object::TValue::get_number_value [/home/jice/lua-rs/target/debug/examples/spectral]
  102,937,056 ( 2.06%)  src/opcodes.rs:lua_rs::opcodes::<impl core::convert::TryFrom<u32> for lua_rs::opcodes::unformatted::OpCode>::try_from [/home/jice/lua-rs/target/debug/examples/spectral]
   99,645,391 ( 1.99%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/slice/index.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index
   95,692,058 ( 1.91%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut [/home/jice/lua-rs/target/debug/examples/spectral]
   90,070,546 ( 1.80%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/result.rs:core::result::Result<T,E>::unwrap [/home/jice/lua-rs/target/debug/examples/spectral]
   72,008,020 ( 1.44%)  src/object.rs:lua_rs::object::TValue::is_number [/home/jice/lua-rs/target/debug/examples/spectral]
   70,812,052 ( 1.42%)  src/ldo.rs:lua_rs::ldo::<impl lua_rs::state::LuaState>::dprecall [/home/jice/lua-rs/target/debug/examples/spectral]
   65,641,892 ( 1.31%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::extend_with [/home/jice/lua-rs/target/debug/examples/spectral]
   58,805,418 ( 1.18%)  src/state.rs:lua_rs::state::LuaState::poscall [/home/jice/lua-rs/target/debug/examples/spectral]
   57,902,094 ( 1.16%)  src/opcodes.rs:lua_rs::opcodes::get_opcode [/home/jice/lua-rs/target/debug/examples/spectral]
   57,618,859 ( 1.15%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::resize [/home/jice/lua-rs/target/debug/examples/spectral]
   54,681,176 ( 1.09%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/metadata.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut
   53,264,428 ( 1.07%)  src/opcodes.rs:lua_rs::opcodes::RK_IS_K [/home/jice/lua-rs/target/debug/examples/spectral]
   49,822,678 ( 1.00%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/raw_vec.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index
   49,822,678 ( 1.00%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/slice/raw.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index
   46,651,545 ( 0.93%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/const_ptr.rs:lua_rs::vm::<impl lua_rs::state::LuaState>::vexecute
   43,529,292 ( 0.87%)  src/state.rs:lua_rs::state::LuaState::get_lua_constant [/home/jice/lua-rs/target/debug/examples/spectral]
   39,219,796 ( 0.78%)  src/state.rs:lua_rs::state::LuaState::get_tablev [/home/jice/lua-rs/target/debug/examples/spectral]
   34,016,303 ( 0.68%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:alloc::rc::RcInnerPtr::inc_strong [/home/jice/lua-rs/target/debug/examples/spectral]
   34,014,693 ( 0.68%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<[lua_rs::object::TValue]> [/home/jice/lua-rs/target/debug/examples/spectral]
   32,167,660 ( 0.64%)  src/opcodes.rs:lua_rs::opcodes::get_arg_a [/home/jice/lua-rs/target/debug/examples/spectral]
   30,045,685 ( 0.60%)  src/opcodes.rs:lua_rs::opcodes::get_arg_b [/home/jice/lua-rs/target/debug/examples/spectral]
   30,012,981 ( 0.60%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::ops::drop::Drop>::drop [/home/jice/lua-rs/target/debug/examples/spectral]
   29,690,432 ( 0.59%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/range.rs:<core::ops::range::Range<T> as core::iter::range::RangeIteratorImpl>::spec_next [/home/jice/lua-rs/target/debug/examples/spectral]
   28,814,904 ( 0.58%)  src/table.rs:lua_rs::table::Table::get [/home/jice/lua-rs/target/debug/examples/spectral]
   27,340,588 ( 0.55%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/slice/index.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut
   24,911,339 ( 0.50%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/const_ptr.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index
   24,911,339 ( 0.50%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/unique.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index
   22,023,830 ( 0.44%)  src/opcodes.rs:lua_rs::opcodes::get_arg_c [/home/jice/lua-rs/target/debug/examples/spectral]
   22,009,174 ( 0.44%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:alloc::rc::RcInnerPtr::dec_strong [/home/jice/lua-rs/target/debug/examples/spectral]
   21,603,726 ( 0.43%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::truncate [/home/jice/lua-rs/target/debug/examples/spectral]
   20,009,590 ( 0.40%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::clone::Clone>::clone [/home/jice/lua-rs/target/debug/examples/spectral]
   19,346,352 ( 0.39%)  src/object.rs:lua_rs::object::Closure::get_proto_id [/home/jice/lua-rs/target/debug/examples/spectral]
   19,262,280 ( 0.39%)  ./string/../sysdeps/x86_64/multiarch/memmove-vec-unaligned-erms.S:__memcpy_avx_unaligned_erms [/usr/lib/x86_64-linux-gnu/libc.so.6]
   16,012,240 ( 0.32%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/raw_vec.rs:alloc::raw_vec::RawVec<T,A>::needs_to_grow [/home/jice/lua-rs/target/debug/examples/spectral]
   16,007,172 ( 0.32%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:alloc::rc::RcInnerPtr::strong [/home/jice/lua-rs/target/debug/examples/spectral]
   14,461,860 ( 0.29%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:lua_rs::vm::<impl lua_rs::state::LuaState>::vexecute
   13,670,294 ( 0.27%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/raw_vec.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut
   13,670,294 ( 0.27%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/slice/raw.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut
   12,836,264 ( 0.26%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cmp.rs:core::cmp::impls::<impl core::cmp::PartialOrd for usize>::lt [/home/jice/lua-rs/target/debug/examples/spectral]
   12,002,100 ( 0.24%)  src/state.rs:<lua_rs::state::CallInfo as core::default::Default>::default [/home/jice/lua-rs/target/debug/examples/spectral]
   12,002,070 ( 0.24%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/metadata.rs:alloc::vec::Vec<T,A>::truncate
   11,728,296 ( 0.23%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:core::cell::RefCell<T>::try_borrow_mut [/home/jice/lua-rs/target/debug/examples/spectral]
   11,208,239 ( 0.22%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/range.rs:alloc::vec::Vec<T,A>::extend_with
   11,201,776 ( 0.22%)  src/state.rs:lua_rs::state::LuaState::close_func [/home/jice/lua-rs/target/debug/examples/spectral]
   10,405,183 ( 0.21%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::pop [/home/jice/lua-rs/target/debug/examples/spectral]
   10,004,370 ( 0.20%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/non_null.rs:<alloc::rc::Rc<T> as core::ops::drop::Drop>::drop
    9,607,968 ( 0.19%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::reserve [/home/jice/lua-rs/target/debug/examples/spectral]
    9,601,656 ( 0.19%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mut_ptr.rs:alloc::vec::Vec<T,A>::truncate
    9,229,294 ( 0.18%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::push [/home/jice/lua-rs/target/debug/examples/spectral]
    8,800,880 ( 0.18%)  src/object.rs:lua_rs::object::Closure::get_lua_upvalue [/home/jice/lua-rs/target/debug/examples/spectral]
    8,493,192 ( 0.17%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/result.rs:core::result::Result<T,E>::expect [/home/jice/lua-rs/target/debug/examples/spectral]
    8,005,550 ( 0.16%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mut_ptr.rs:alloc::vec::Vec<T,A>::extend_with
    8,004,792 ( 0.16%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:alloc::rc::Rc<T>::from_inner [/home/jice/lua-rs/target/debug/examples/spectral]
    8,003,586 ( 0.16%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:alloc::rc::RcInnerPtr::strong
    8,001,350 ( 0.16%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<alloc::rc::Rc<lua_rs::object::Closure>> [/home/jice/lua-rs/target/debug/examples/spectral]
    7,204,794 ( 0.14%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/set_len_on_drop.rs:alloc::vec::Vec<T,A>::extend_with
    6,835,147 ( 0.14%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mut_ptr.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut
    6,835,147 ( 0.14%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/non_null.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut
    6,835,147 ( 0.14%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/unique.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut
    6,406,048 ( 0.13%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::ExtendElement<T> as alloc::vec::ExtendWith<T>>::next [/home/jice/lua-rs/target/debug/examples/spectral]
    6,404,708 ( 0.13%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:alloc::vec::Vec<T,A>::extend_with
    6,066,360 ( 0.12%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:core::cell::BorrowRefMut::new [/home/jice/lua-rs/target/debug/examples/spectral]
    6,002,877 ( 0.12%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/num/uint_macros.rs:alloc::rc::RcInnerPtr::inc_strong
    6,002,740 ( 0.12%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/slice/mod.rs:core::slice::<impl [T]>::last [/home/jice/lua-rs/target/debug/examples/spectral]
    5,661,936 ( 0.11%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:core::cell::RefCell<T>::borrow_mut [/home/jice/lua-rs/target/debug/examples/spectral]
    5,624,728 ( 0.11%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/non_null.rs:lua_rs::vm::<impl lua_rs::state::LuaState>::vexecute
    5,257,512 ( 0.11%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:<core::cell::BorrowRefMut as core::ops::drop::Drop>::drop [/home/jice/lua-rs/target/debug/examples/spectral]
    4,899,204 ( 0.10%)  src/opcodes.rs:lua_rs::opcodes::get_arg_sbx [/home/jice/lua-rs/target/debug/examples/spectral]
    4,802,526 ( 0.10%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::ExtendElement<T> as alloc::vec::ExtendWith<T>>::last [/home/jice/lua-rs/target/debug/examples/spectral]
    4,013,170 ( 0.08%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/raw_vec.rs:alloc::vec::Vec<T,A>::push
    4,002,105 ( 0.08%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/set_len_on_drop.rs:<alloc::vec::set_len_on_drop::SetLenOnDrop as core::ops::drop::Drop>::drop [/home/jice/lua-rs/target/debug/examples/spectral]
    4,002,105 ( 0.08%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<alloc::vec::set_len_on_drop::SetLenOnDrop> [/home/jice/lua-rs/target/debug/examples/spectral]
    4,001,918 ( 0.08%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:alloc::rc::RcInnerPtr::inc_strong
    4,001,918 ( 0.08%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/mem/mod.rs:alloc::rc::RcInnerPtr::inc_strong
    4,001,918 ( 0.08%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:alloc::rc::RcInnerPtr::inc_strong
    4,001,918 ( 0.08%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/non_null.rs:<alloc::rc::Rc<T> as core::clone::Clone>::clone
    4,001,668 ( 0.08%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:alloc::rc::RcInnerPtr::dec_strong
    4,001,668 ( 0.08%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/mem/mod.rs:alloc::rc::RcInnerPtr::dec_strong
    4,001,668 ( 0.08%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:alloc::rc::RcInnerPtr::dec_strong
    4,000,780 ( 0.08%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<usize as core::default::Default>::default [/home/jice/lua-rs/target/debug/examples/spectral]
    3,616,173 ( 0.07%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len [/home/jice/lua-rs/target/debug/examples/spectral]
    3,213,928 ( 0.06%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:<core::ops::range::Range<T> as core::iter::range::RangeIteratorImpl>::spec_next
    3,202,832 ( 0.06%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/metadata.rs:<alloc::vec::Vec<T,A> as core::ops::deref::Deref>::deref
    2,807,691 ( 0.06%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mut_ptr.rs:alloc::vec::Vec<T,A>::push
    2,426,532 ( 0.05%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<core::cell::RefMut<lua_rs::table::Table>> [/home/jice/lua-rs/target/debug/examples/spectral]
    2,410,524 ( 0.05%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/range.rs:<usize as core::iter::range::Step>::forward_unchecked [/home/jice/lua-rs/target/debug/examples/spectral]
    2,410,524 ( 0.05%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/num/uint_macros.rs:<usize as core::iter::range::Step>::forward_unchecked
    2,410,446 ( 0.05%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/mem/mod.rs:<core::ops::range::Range<T> as core::iter::range::RangeIteratorImpl>::spec_next
    2,402,252 ( 0.05%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/raw_vec.rs:alloc::vec::Vec<T,A>::reserve
    2,402,124 ( 0.05%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::deref::Deref>::deref [/home/jice/lua-rs/target/debug/examples/spectral]
    2,401,296 ( 0.05%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/const_ptr.rs:alloc::vec::Vec<T,A>::pop
    2,400,414 ( 0.05%)  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/raw_vec.rs:alloc::vec::Vec<T,A>::truncate

--------------------------------------------------------------------------------
-- Auto-annotated source: src/opcodes.rs
--------------------------------------------------------------------------------
Ir                  

-- line 33 ----------------------------------------
         .           /// this bit 1 means constant (0 means register)
         .           pub const BIT_RK: u32 = 1 << (SIZE_B - 1);
         .           pub const MAX_INDEX_RK: usize = BIT_RK as usize - 1;
         .           
         .           /// number of list items to accumulate before a SETLIST instruction
         .           pub const LFIELDS_PER_FLUSH: i32 = 50;
         .           
         .           #[inline]
        14 ( 0.00%)  pub(crate) const fn RK_AS_K(val: u32) -> u32 {
         7 ( 0.00%)      val | BIT_RK
         7 ( 0.00%)  }
         .           #[inline]
 7,609,204 ( 0.15%)  pub(crate) const fn RK_IS_K(val: u32) -> bool {
22,827,612 ( 0.46%)      val & BIT_RK != 0
22,827,612 ( 0.46%)  }
         .           
         .           pub const MAXARG_A: usize = (1 << SIZE_A) - 1;
         .           pub const MAXARG_B: usize = (1 << SIZE_B) - 1;
         .           pub const MAXARG_C: usize = (1 << SIZE_C) - 1;
         .           pub const MAXARG_BX: usize = (1 << SIZE_BX) - 1;
         .           pub const MAXARG_SBX: i32 = (MAXARG_BX >> 1) as i32;
         .           /// value for an invalid register
         .           pub const NO_REG: u32 = MAXARG_A as u32;
-- line 55 ----------------------------------------
-- line 106 ----------------------------------------
         .           pub const MASK_SET_C: u32 =   0b00000000011111111100000000000000;
         .           pub const MASK_SET_B: u32 =   0b11111111100000000000000000000000;
         .           pub const MASK_UNSET_A: u32 = 0b11111111111111111100000000111111;
         .           pub const MASK_UNSET_C: u32 = 0b11111111100000000011111111111111;
         .           pub const MASK_UNSET_B: u32 = 0b00000000011111111111111111111111;
         .           pub const MASK_SET_BX: u32 =  0b11111111111111111100000000000000;
         .           pub const MASK_UNSET_BX: u32 =0b00000000000000000011111111111111;
         .           
       594 ( 0.00%)  #[derive(PartialEq,Clone,Copy)]
         .           pub enum OpCode {
         .               //----------------------------------------------------------------------
         .               //    		args	description
         .               //name
         .               //----------------------------------------------------------------------
         .               /// 	    A B	    R(A) := R(B)
         .               Move = 0,
         .               /// 	    A Bx	R(A) := Kst(Bx)
-- line 122 ----------------------------------------
-- line 194 ----------------------------------------
         .               /// 	    A B	    R(A), R(A+1), ..., R(A+B-1) = vararg
         .               VarArg
         .           }
         .           
         .           }
         .           pub use unformatted::*;
         .           
         .           impl OpCode {
        14 ( 0.00%)      pub(crate) fn is_test(&self) -> bool {
       126 ( 0.00%)          match self {
         .                       OpCode::Eq
         .                       | OpCode::Lt
         .                       | OpCode::Le
         .                       | OpCode::Test
         .                       | OpCode::TestSet
         .                       | OpCode::TForLoop => true,
        28 ( 0.00%)              _ => false,
         .                   }
        56 ( 0.00%)      }
         .               pub(crate) fn is_abx(&self) -> bool {
         .                   match self {
         .                       OpCode::LoadK | OpCode::GetGlobal | OpCode::SetGlobal | OpCode::Closure => true,
         .                       _ => false,
         .                   }
         .               }
         .               pub(crate) fn is_asbx(&self) -> bool {
         .                   match self {
-- line 220 ----------------------------------------
-- line 222 ----------------------------------------
         .                       _ => false,
         .                   }
         .               }
         .           }
         .           
         .           impl TryFrom<u32> for OpCode {
         .               type Error = ();
         .           
 6,433,566 ( 0.13%)      fn try_from(value: u32) -> Result<Self, Self::Error> {
25,734,264 ( 0.51%)          match value {
 3,217,096 ( 0.06%)              0 => Ok(Self::Move),
    48,392 ( 0.00%)              1 => Ok(Self::LoadK),
         .                       2 => Ok(Self::LoadBool),
         .                       3 => Ok(Self::LoadNil),
 1,600,160 ( 0.03%)              4 => Ok(Self::GetUpVal),
        12 ( 0.00%)              5 => Ok(Self::GetGlobal),
 1,600,808 ( 0.03%)              6 => Ok(Self::GetTable),
         .                       7 => Ok(Self::SetGlobal),
         .                       8 => Ok(Self::SetupVal),
    16,412 ( 0.00%)              9 => Ok(Self::SetTable),
        12 ( 0.00%)              10 => Ok(Self::NewTable),
         .                       11 => Ok(Self::OpSelf),
 4,800,812 ( 0.10%)              12 => Ok(Self::Add),
 3,200,000 ( 0.06%)              13 => Ok(Self::Sub),
 4,800,800 ( 0.10%)              14 => Ok(Self::Mul),
 1,600,004 ( 0.03%)              15 => Ok(Self::Div),
         .                       16 => Ok(Self::Mod),
         .                       17 => Ok(Self::Pow),
         .                       18 => Ok(Self::UnaryMinus),
         .                       19 => Ok(Self::Not),
         .                       20 => Ok(Self::Len),
         .                       21 => Ok(Self::Concat),
         .                       22 => Ok(Self::Jmp),
         .                       23 => Ok(Self::Eq),
         .                       24 => Ok(Self::Lt),
         .                       25 => Ok(Self::Le),
         .                       26 => Ok(Self::Test),
         .                       27 => Ok(Self::TestSet),
 1,600,256 ( 0.03%)              28 => Ok(Self::Call),
         .                       29 => Ok(Self::TailCall),
 1,600,244 ( 0.03%)              30 => Ok(Self::Return),
 1,633,040 ( 0.03%)              31 => Ok(Self::ForLoop),
    16,200 ( 0.00%)              32 => Ok(Self::ForPrep),
         .                       33 => Ok(Self::TForLoop),
         .                       34 => Ok(Self::SetList),
         .                       35 => Ok(Self::Close),
        16 ( 0.00%)              36 => Ok(Self::Closure),
         .                       37 => Ok(Self::VarArg),
         .                       _ => Err(()),
         .                   }
12,867,132 ( 0.26%)      }
         .           }
         .           
         .           #[inline]
12,867,132 ( 0.26%)  pub(crate) fn get_opcode(i: Instruction) -> OpCode {
32,167,830 ( 0.64%)      unsafe { std::mem::transmute((i & MASK_SET_OP) as u8) }
102,937,056 ( 2.06%)  => src/opcodes.rs:lua_rs::opcodes::<impl core::convert::TryFrom<u32> for lua_rs::opcodes::unformatted::OpCode>::try_from (6,433,566x)
90,069,924 ( 1.80%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/result.rs:core::result::Result<T,E>::unwrap (6,433,566x)
12,867,132 ( 0.26%)  }
         .           pub(crate) fn set_opcode(dest: &mut Instruction, arg: u32) {
         .               *dest = (*dest & MASK_UNSET_OP) | (arg & MASK_SET_OP);
         .           }
         .           
         .           #[inline]
12,867,064 ( 0.26%)  pub(crate) fn get_arg_a(i: Instruction) -> u32 {
12,867,064 ( 0.26%)      (i & MASK_SET_A) >> POS_A
 6,433,532 ( 0.13%)  }
       108 ( 0.00%)  pub(crate) fn set_arg_a(dest: &mut Instruction, arg: u32) {
       216 ( 0.00%)      *dest = (*dest & MASK_UNSET_A) | ((arg << POS_A) & MASK_SET_A);
        36 ( 0.00%)  }
         .           
         .           #[inline]
12,018,274 ( 0.24%)  pub(crate) fn get_arg_b(i: Instruction) -> u32 {
12,018,274 ( 0.24%)      (i & MASK_SET_B) >> POS_B
 6,009,137 ( 0.12%)  }
         9 ( 0.00%)  pub(crate) fn set_arg_b(dest: &mut Instruction, arg: u32) {
        18 ( 0.00%)      *dest = (*dest & MASK_UNSET_B) | ((arg << POS_B) & MASK_SET_B);
         3 ( 0.00%)  }
         .           
         .           #[inline]
 8,809,532 ( 0.18%)  pub(crate) fn get_arg_c(i: Instruction) -> u32 {
 8,809,532 ( 0.18%)      (i & MASK_SET_C) >> POS_C
 4,404,766 ( 0.09%)  }
        30 ( 0.00%)  pub(crate) fn set_arg_c(dest: &mut Instruction, arg: u32) {
        60 ( 0.00%)      *dest = (*dest & MASK_UNSET_C) | ((arg << POS_C) & MASK_SET_C);
        10 ( 0.00%)  }
         .           
         .           #[inline]
   840,730 ( 0.02%)  pub(crate) fn get_arg_bx(i: Instruction) -> u32 {
   840,730 ( 0.02%)      (i & MASK_SET_BX) >> POS_BX
   420,365 ( 0.01%)  }
        42 ( 0.00%)  pub(crate) fn set_arg_bx(dest: &mut Instruction, arg: u32) {
        84 ( 0.00%)      *dest = (*dest & MASK_UNSET_BX) | ((arg << POS_BX) & MASK_SET_BX);
        14 ( 0.00%)  }
         .           
         .           #[inline]
   816,534 ( 0.02%)  pub(crate) fn get_arg_sbx(i: Instruction) -> i32 {
 2,857,869 ( 0.06%)      (get_arg_bx(i) as i64 - MAXARG_SBX as i64) as i32
 2,041,335 ( 0.04%)  => src/opcodes.rs:lua_rs::opcodes::get_arg_bx (408,267x)
   816,534 ( 0.02%)  }
        56 ( 0.00%)  pub(crate) fn set_arg_sbx(dest: &mut Instruction, sbx: i32) {
        84 ( 0.00%)      set_arg_bx(dest, (sbx + MAXARG_SBX) as u32);
       140 ( 0.00%)  => src/opcodes.rs:lua_rs::opcodes::set_arg_bx (14x)
        28 ( 0.00%)  }
         .           
       385 ( 0.00%)  pub(crate) fn create_abc(opcode: u32, a: i32, b: i32, c: i32) -> u32 {
       231 ( 0.00%)      opcode
       154 ( 0.00%)          | ((a << POS_A) as u32 & MASK_SET_A)
       154 ( 0.00%)          | ((b << POS_B) as u32 & MASK_SET_B)
       154 ( 0.00%)          | ((c << POS_C) as u32 & MASK_SET_C)
        77 ( 0.00%)  }
         .           
       252 ( 0.00%)  pub(crate) fn create_abx(opcode: u32, a: i32, bx: u32) -> u32 {
       252 ( 0.00%)      opcode | ((a << POS_A) as u32 & MASK_SET_A) | ((bx << POS_BX) & MASK_SET_BX)
        42 ( 0.00%)  }
         .           
        71 ( 0.00%)  pub(crate) fn is_reg_constant(reg: u32) -> bool {
       213 ( 0.00%)      reg & BIT_RK != 0
       213 ( 0.00%)  }

32,576,139 ( 0.65%)  <counts for unidentified lines in src/opcodes.rs>

--------------------------------------------------------------------------------
-- Auto-annotated source: src/table.rs
--------------------------------------------------------------------------------
Ir                 

-- line 6 ----------------------------------------
        .           //! part.
        .           
        .           use std::{collections::HashMap, cell::RefCell, rc::Rc};
        .           
        .           use crate::object::TValue;
        .           
        .           pub type TableRef = Rc<RefCell<Table>>;
        .           
       36 ( 0.00%)  #[derive(Clone, Default)]
       26 ( 0.00%)  => ???:0x000000000011a060 (2x)
        .           pub struct Table {
        4 ( 0.00%)      pub flags: u8,
        4 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<u8 as core::default::Default>::default (2x)
        6 ( 0.00%)      pub metatable: Option<TableRef>,
        6 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/option.rs:<core::option::Option<T> as core::default::Default>::default (2x)
        8 ( 0.00%)      pub array: Vec<TValue>,
       14 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T> as core::default::Default>::default (2x)
        8 ( 0.00%)      pub node: HashMap<TValue, TValue>,
      472 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/std/src/collections/hash/map.rs:<std::collections::hash::map::HashMap<K,V,S> as core::default::Default>::default (2x)
        .           }
        .           
        .           impl Table {
       33 ( 0.00%)      pub fn new() -> Self {
      143 ( 0.00%)          Self {
      143 ( 0.00%)  => ???:0x000000000011a060 (11x)
        .                       flags: !0,
       11 ( 0.00%)              metatable: None,
       44 ( 0.00%)              array: Vec::new(),
       77 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T>::new (11x)
       44 ( 0.00%)              node: HashMap::new(),
    2,699 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/std/src/collections/hash/map.rs:std::collections::hash::map::HashMap<K,V>::new (11x)
        .                   }
       22 ( 0.00%)      }
        .           
   21,000 ( 0.00%)      pub fn set(&mut self, key: TValue, value: TValue) {
   33,600 ( 0.00%)          match key {
   41,000 ( 0.00%)              TValue::Number(n) if n >= 1.0 => {
   77,900 ( 0.00%)                  let n = n as usize;
   28,700 ( 0.00%)                  if n > self.array.len() {
   12,300 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (4,100x)
    2,100 ( 0.00%)                      self.array.resize(n, TValue::Nil);
   77,949 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::resize (300x)
        .                           }
  123,000 ( 0.00%)                  self.array[n-1] = value;
  209,100 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut (4,100x)
   57,400 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (4,100x)
    4,100 ( 0.00%)              }
        .                       _ => {
    1,600 ( 0.00%)                  self.node.insert(key, value);
  397,347 ( 0.01%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/std/src/collections/hash/map.rs:std::collections::hash::map::HashMap<K,V,S>::insert (100x)
    1,100 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<core::option::Option<lua_rs::object::TValue>> (100x)
        .                       }
        .                   }
   25,100 ( 0.00%)      }
   57,400 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (4,100x)
2,001,105 ( 0.04%)      pub fn get(&mut self, key: &TValue) -> Option<&TValue> {
2,801,610 ( 0.06%)          match *key {
        .                       TValue::Nil => return Some(&TValue::Nil),
4,002,000 ( 0.08%)              TValue::Number(n) if n >= 1.0 => {
7,603,800 ( 0.15%)                  let n = n as usize;
2,401,200 ( 0.05%)                  if n > self.array.len() {
1,200,600 ( 0.02%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (400,200x)
        .                               self.array.resize(n, TValue::Nil);
        .                           }
5,202,600 ( 0.10%)                  Some(&self.array[n-1])
20,410,200 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (400,200x)
  400,200 ( 0.01%)              }
       42 ( 0.00%)              TValue::String(_) => self.node.get(key),
   28,390 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/std/src/collections/hash/map.rs:std::collections::hash::map::HashMap<K,V,S>::get (21x)
        .                       _ => todo!(),
        .                   }
1,200,663 ( 0.02%)      }
        .           }
        .           
        .           
        .           
        .           #[cfg(test)]
        .           mod tests {
        .               use crate::{luaH, object::TValue};
        .               #[test]
-- line 67 ----------------------------------------

3,255,899 ( 0.07%)  <counts for unidentified lines in src/table.rs>

--------------------------------------------------------------------------------
-- Auto-annotated source: src/object.rs
--------------------------------------------------------------------------------
Ir                   

-- line 10 ----------------------------------------
          .           
          .           /// index in the current stack
          .           pub type StkId = usize;
          .           
          .           pub type UserDataRef = Rc<RefCell<UserData>>;
          .           /// index in the LuaState.protos vector
          .           pub type ProtoRef = usize;
          .           
124,464,291 ( 2.49%)  #[derive(Clone, Default)]
          .           pub enum TValue {
          .               #[default]
          2 ( 0.00%)      Nil,
 36,950,880 ( 0.74%)      Number(LuaNumber),
        195 ( 0.00%)      String(Rc<String>),
      1,104 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::clone::Clone>::clone (23x)
        925 ( 0.00%)      Table(TableRef),
      8,400 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::clone::Clone>::clone (175x)
  2,000,375 ( 0.04%)      Function(Rc<Closure>),
 19,203,600 ( 0.38%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::clone::Clone>::clone (400,075x)
          .               Boolean(bool),
          .               UserData(UserDataRef),
          .               Thread(),
          .               LightUserData(),
          .           }
          .           
          .           pub(crate) const TVALUE_TYPE_NAMES: [&str; 8] = [
          .               "nil", "number", "string", "table", "function", "userdata", "thread", "userdata",
          .           ];
          .           
          .           pub const TVALUE_TYPE_COUNT: usize = 9;
          .           
          .           impl TValue {
          1 ( 0.00%)      pub fn get_lua_type(&self) -> LuaType {
          2 ( 0.00%)          match self {
          .                       TValue::Boolean(_) => LuaType::Boolean,
          .                       TValue::Nil => LuaType::Nil,
          .                       TValue::Number(_) => LuaType::Number,
          2 ( 0.00%)              TValue::String(_) => LuaType::String,
          .                       TValue::Table(_) => LuaType::Table,
          .                       TValue::Function(_) => LuaType::Function,
          .                       TValue::UserData(_) => LuaType::UserData,
          .                       TValue::Thread() => LuaType::Thread,
          .                       TValue::LightUserData() => LuaType::LightUserData,
          .                   }
          2 ( 0.00%)      }
          .               pub fn get_lua_closure(&self) -> &LClosure {
          .                   if let TValue::Function(cl) = self {
          .                       if let Closure::Lua(luacl) = cl.as_ref() {
          .                           return luacl;
          .                       }
          .                   }
          .                   unreachable!()
          .               }
-- line 59 ----------------------------------------
-- line 69 ----------------------------------------
          .                       TValue::Table(_) => 3,
          .                       TValue::Function(_) => 4,
          .                       TValue::Boolean(_) => 5,
          .                       TValue::UserData(_) => 6,
          .                       TValue::Thread() => 7,
          .                       TValue::LightUserData() => 8,
          .                   }
          .               }
         80 ( 0.00%)      pub fn new_string(val: &str) -> Self {
        128 ( 0.00%)          Self::String(Rc::new(val.to_owned()))
      7,359 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/str.rs:alloc::str::<impl alloc::borrow::ToOwned for str>::to_owned (16x)
      4,778 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:alloc::rc::Rc<T>::new (16x)
         80 ( 0.00%)  => src/object.rs:lua_rs::object::TValue::String (16x)
         32 ( 0.00%)      }
         30 ( 0.00%)      pub fn new_table() -> Self {
        110 ( 0.00%)          Self::Table(Rc::new(RefCell::new(Table::new())))
      3,304 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:alloc::rc::Rc<T>::new (10x)
      2,948 ( 0.00%)  => /home/jice/lua-rs/src/table.rs:lua_rs::table::Table::new (10x)
        610 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:core::cell::RefCell<T>::new (10x)
         50 ( 0.00%)  => src/object.rs:lua_rs::object::TValue::Table (10x)
         20 ( 0.00%)      }
         11 ( 0.00%)      pub fn is_nil(&self) -> bool {
         33 ( 0.00%)          match self {
         10 ( 0.00%)              TValue::Nil => true,
          6 ( 0.00%)              _ => false,
          .                   }
         44 ( 0.00%)      }
 16,867,298 ( 0.34%)      pub fn get_number_value(&self) -> LuaNumber {
 25,300,947 ( 0.51%)          match self {
 50,601,894 ( 1.01%)              TValue::Number(n) => *n,
          .                       _ => 0.0,
          .                   }
 16,867,298 ( 0.34%)      }
  7,200,802 ( 0.14%)      pub fn is_number(&self) -> bool {
 21,602,406 ( 0.43%)          match self {
 14,401,604 ( 0.29%)              TValue::Number(_) => true,
          .                       _ => false,
          .                   }
 28,803,208 ( 0.58%)      }
          .               pub fn is_string(&self) -> bool {
          .                   match self {
          .                       TValue::String(_) => true,
          .                       _ => false,
          .                   }
          .               }
         10 ( 0.00%)      pub fn is_table(&self) -> bool {
         30 ( 0.00%)          match self {
         10 ( 0.00%)              TValue::Table(_) => true,
          5 ( 0.00%)              _ => false,
          .                   }
         40 ( 0.00%)      }
          .               pub fn is_function(&self) -> bool {
          .                   match self {
          .                       TValue::Function(_) => true,
          .                       _ => false,
          .                   }
          .               }
          .               pub fn is_boolean(&self) -> bool {
          .                   match self {
-- line 120 ----------------------------------------
-- line 153 ----------------------------------------
          .                   match self {
          .                       TValue::String(s) => write!(f, "{:?}", s),
          .                       _ => write!(f, "{}", self),
          .                   }
          .               }
          .           }
          .           
          .           impl PartialEq for TValue {
        760 ( 0.00%)      fn eq(&self, other: &Self) -> bool {
      2,002 ( 0.00%)          match (self, other) {
        504 ( 0.00%)              (Self::Number(l0), Self::Number(r0)) => l0 == r0,
        840 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cmp.rs:core::cmp::impls::<impl core::cmp::PartialEq<&B> for &A>::eq (42x)
        204 ( 0.00%)              (Self::String(l0), Self::String(r0)) => l0 == r0,
      3,336 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cmp.rs:core::cmp::impls::<impl core::cmp::PartialEq<&B> for &A>::eq (17x)
          .                       (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
      1,023 ( 0.00%)              _ => core::mem::discriminant(self) == core::mem::discriminant(other),
      1,488 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/mem/mod.rs:<core::mem::Discriminant<T> as core::cmp::PartialEq>::eq (93x)
        930 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/mem/mod.rs:core::mem::discriminant (186x)
          .                   }
        760 ( 0.00%)      }
          .           }
          .           
          .           impl Eq for TValue {}
          .           
          .           impl std::hash::Hash for TValue {
      1,580 ( 0.00%)      fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        948 ( 0.00%)          match self {
      1,112 ( 0.00%)              TValue::String(s) => s.hash(state),
     89,938 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::hash::Hash>::hash (278x)
        190 ( 0.00%)              _ => core::mem::discriminant(self).hash(state),
      8,360 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/mem/mod.rs:<core::mem::Discriminant<T> as core::hash::Hash>::hash (38x)
        190 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/mem/mod.rs:core::mem::discriminant (38x)
          .                   }
        632 ( 0.00%)      }
          .           }
          .           
          .           #[derive(Clone, Default)]
          .           pub struct UserData {
          .               pub metatable: Option<TableRef>,
          .               pub env: Option<TableRef>,
          .           }
          .           
        825 ( 0.00%)  #[derive(Clone, Default)]
          .           pub struct LocVar {
        330 ( 0.00%)      pub name: String,
     10,702 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/string.rs:<alloc::string::String as core::clone::Clone>::clone (55x)
          .               /// first point where variable is active
         55 ( 0.00%)      pub start_pc: usize,
          .               /// first point where variable is dead
         55 ( 0.00%)      pub end_pc: usize,
          .           }
          .           
        490 ( 0.00%)  #[derive(Clone, Default)]
          .           pub struct Proto {
          .               /// constants used by the function
         40 ( 0.00%)      pub k: Vec<TValue>,
         35 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T> as core::default::Default>::default (5x)
          .               /// the bytecode
         45 ( 0.00%)      pub code: Vec<Instruction>,
         35 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T> as core::default::Default>::default (5x)
          .               /// functions defined inside the function
         45 ( 0.00%)      pub p: Vec<ProtoRef>,
         35 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T> as core::default::Default>::default (5x)
          .               /// map from opcodes to source lines
         45 ( 0.00%)      pub lineinfo: Vec<usize>,
         35 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T> as core::default::Default>::default (5x)
          .               /// information about local variables
         45 ( 0.00%)      pub locvars: Vec<LocVar>,
         35 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T> as core::default::Default>::default (5x)
          .               /// number of upvalues
         25 ( 0.00%)      pub nups: usize,
         10 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<usize as core::default::Default>::default (5x)
         25 ( 0.00%)      pub linedefined: usize,
         10 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<usize as core::default::Default>::default (5x)
         25 ( 0.00%)      pub lastlinedefined: usize,
         10 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<usize as core::default::Default>::default (5x)
         25 ( 0.00%)      pub numparams: usize,
         10 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<usize as core::default::Default>::default (5x)
         25 ( 0.00%)      pub is_vararg: bool,
         20 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<bool as core::default::Default>::default (5x)
         25 ( 0.00%)      pub maxstacksize: usize,
         10 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<usize as core::default::Default>::default (5x)
          .               /// file name
         35 ( 0.00%)      pub source: String,
         65 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/string.rs:<alloc::string::String as core::default::Default>::default (5x)
          .           }
          .           
          .           impl Proto {
         30 ( 0.00%)      pub fn new(source: &str) -> Self {
        245 ( 0.00%)          Self {
         10 ( 0.00%)              source:source.to_owned(),
      3,416 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/str.rs:alloc::str::<impl alloc::borrow::ToOwned for str>::to_owned (5x)
         10 ( 0.00%)              ..Self::default()
        790 ( 0.00%)  => src/object.rs:<lua_rs::object::Proto as core::default::Default>::default (5x)
          .                   }
         30 ( 0.00%)      }
        355 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<alloc::string::String> (5x)
          .           }
          .           
         44 ( 0.00%)  #[derive(Clone, Default)]
          .           pub struct UpVal {
          8 ( 0.00%)      pub v: StkId,
         20 ( 0.00%)      pub value: TValue,
        288 ( 0.00%)  => src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (4x)
          .           }
          .           
          .           /// native rust closure
          .           #[derive(Clone)]
          .           pub struct RClosure {
          .               pub f: LuaRustFunction,
          .               pub upvalues: Vec<TValue>,
          .               pub env: TableRef,
          .               pub envvalue: TValue,
          .           }
          .           
          .           impl RClosure {
        546 ( 0.00%)      pub fn new(func: LuaRustFunction, env: TableRef) -> Self {
        637 ( 0.00%)          let envvalue = TValue::Table(Rc::clone(&env));
      4,368 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::clone::Clone>::clone (91x)
      1,092 ( 0.00%)          Self {
          .                       f: func,
        364 ( 0.00%)              upvalues: Vec::new(),
        637 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T>::new (91x)
         91 ( 0.00%)              env,
        364 ( 0.00%)              envvalue,
          .                   }
        182 ( 0.00%)      }
          .               pub fn borrow_upvalue(&self, index: usize) -> &TValue {
          .                   &self.upvalues[index]
          .               }
          .           }
          .           
          .           /// Lua closure
          .           #[derive(Clone)]
          .           pub struct LClosure {
          .               pub proto: ProtoRef,
          .               pub upvalues: Vec<UpVal>,
          .               pub env: TableRef,
          .               pub envvalue: TValue,
          .           }
          .           
          .           impl LClosure {
         30 ( 0.00%)      pub fn new(proto: ProtoRef, env: TableRef) -> Self {
         35 ( 0.00%)          let envvalue = TValue::Table(Rc::clone(&env));
        240 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::clone::Clone>::clone (5x)
         60 ( 0.00%)          Self {
          .                       proto,
         20 ( 0.00%)              upvalues: Vec::new(),
         35 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T>::new (5x)
          5 ( 0.00%)              env,
         20 ( 0.00%)              envvalue,
          .                   }
         10 ( 0.00%)      }
          .           }
          .           
          .           #[derive(Clone)]
          .           pub enum Closure {
          .               Rust(RClosure),
          .               Lua(LClosure),
          .           }
          .           
          .           impl Closure {
        261 ( 0.00%)      pub fn get_env(&self) -> TableRef {
        174 ( 0.00%)          match self {
        522 ( 0.00%)              Closure::Rust(cl) => Rc::clone(&cl.env),
      4,176 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::clone::Clone>::clone (87x)
          .                       Closure::Lua(cl) => Rc::clone(&cl.env),
          .                   }
        261 ( 0.00%)      }
          .           
          .               #[inline]
  2,800,280 ( 0.06%)      pub fn get_lua_upvalue(&self, id: usize) -> TValue {
  1,600,160 ( 0.03%)          if let Closure::Lua(cl) = self {
  2,800,280 ( 0.06%)              return cl.upvalues[id].value.clone();
 28,802,880 ( 0.58%)  => src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (400,040x)
 20,402,040 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (400,040x)
          .                   }
          .                   unreachable!()
    800,080 ( 0.02%)      }
  4,836,588 ( 0.10%)      #[inline]
  3,224,392 ( 0.06%)      pub fn get_proto_id(&self) -> usize {
          .                   match self {
  6,448,784 ( 0.13%)              Closure::Rust(_cl) => unreachable!(),
          .                       Closure::Lua(cl) => cl.proto,
  3,224,392 ( 0.06%)          }
          6 ( 0.00%)      }
          6 ( 0.00%)      pub fn get_envvalue(&self) -> &TValue {
          .                   match self {
          9 ( 0.00%)              Closure::Rust(cl) => &cl.envvalue,
          .                       Closure::Lua(cl) => &cl.envvalue,
          6 ( 0.00%)          }
          .               }
          .               pub fn get_nupvalues(&self) -> usize {
          .                   match self {
          .                       Closure::Rust(cl) => cl.upvalues.len(),
          .                       Closure::Lua(cl) => cl.upvalues.len(),
          .                   }
          .               }
          .           }
-- line 320 ----------------------------------------
-- line 367 ----------------------------------------
          .                   let v = h.get(&TValue::new_string("key"));
          .           
          .                   assert_eq!(v, Some(&123));
          .               }
          .           }
          .           
          .           /// converts an integer to a "floating point byte", represented as
          .           /// (eeeeexxx), where the real value is (1xxx) * 2^(eeeee - 1) if
         12 ( 0.00%)  /// eeeee != 0 and (xxx) otherwise.
          6 ( 0.00%)  pub(crate) const fn INT2FB(val: u32) -> u32 {
          6 ( 0.00%)      let mut e = 0; // exponent
         12 ( 0.00%)      let mut val = val;
          .               while val >= 16 {
          .                   val = (val + 1) >> 1;
          .                   e += 1;
         12 ( 0.00%)      }
         12 ( 0.00%)      if val < 8 {
          .                   val
          .               } else {
          .                   ((e + 1) << 3) | (val - 8)
         18 ( 0.00%)      }
          .           }

 93,156,402 ( 1.86%)  <counts for unidentified lines in src/object.rs>

--------------------------------------------------------------------------------
-- Auto-annotated source: src/vm.rs
--------------------------------------------------------------------------------
Ir                   

-- line 36 ----------------------------------------
          .                           todo!();
          .                           $base = $state.base as u32;
          .                       }
          .                   }
          .               }
          .           }
          .           
          .           impl LuaState {
          4 ( 0.00%)      pub(crate) fn vexecute(&mut self, nexec_calls: i32) -> Result<(), LuaError> {
         20 ( 0.00%)          let mut nexec_calls = nexec_calls;
          .                   'reentry: loop {
  6,400,968 ( 0.13%)              let func = self.base_ci[self.ci].func;
 40,806,171 ( 0.82%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (800,121x)
  1,600,242 ( 0.03%)              let mut pc = self.saved_pc;
  7,201,089 ( 0.14%)              let cl = if let TValue::Function(cl) = &self.stack[func] {
 40,806,171 ( 0.82%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (800,121x)
  3,200,484 ( 0.06%)                  cl.clone()
 38,405,808 ( 0.77%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::clone::Clone>::clone (800,121x)
          .                       } else {
          .                           unreachable!()
          .                       };
  4,000,605 ( 0.08%)              let protoid = if let Closure::Lua(cl_lua) = &*cl {
  1,600,242 ( 0.03%)                  cl_lua.proto
          .                       } else {
          .                           unreachable!()
          .                       };
  1,600,242 ( 0.03%)              let mut base = self.base as u32;
 12,001,815 ( 0.24%)              let mut pcptr=unsafe {self.protos[protoid].code.as_ptr().add(pc)};
 40,806,171 ( 0.82%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (800,121x)
  4,800,726 ( 0.10%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::as_ptr (800,121x)
          .                       #[cfg(feature="debug_logs")] let mut first=true;
          .                       // main loop of interpreter
          .                       loop {
 70,768,830 ( 1.42%)                  let i = unsafe {*pcptr};//self.protos[protoid].code[pc];
          .                           #[cfg(feature="debug_logs")] 
          .                           {
          .                               if let Closure::Lua(cl_lua) = &*cl {
          .                                   if first {dump_function_header(self, cl_lua);first=false;}
          .                               } else {
          .                                   unreachable!()
          .                               };
          .                               if let Closure::Lua(cl_lua) = &*cl {
          .                                   debug_println!("[{:04x}] {}",pc,&disassemble(self,i,cl_lua));
          .                               }
          .                           }
 51,468,240 ( 1.03%)                  pc += 1;
 25,734,120 ( 0.51%)                  unsafe {pcptr = pcptr.add(1);}
          .                           // TODO handle hooks
 70,768,830 ( 1.42%)                  let ra = base + get_arg_a(i);
 32,167,650 ( 0.64%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_a (6,433,530x)
 51,468,240 ( 1.03%)                  debug_assert!(
109,370,010 ( 2.19%)                      self.base == base as usize && self.base == self.base_ci[self.ci].base
328,110,030 ( 6.57%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (6,433,530x)
          .                           );
 38,601,180 ( 0.77%)                  match get_opcode(i) {
250,907,670 ( 5.02%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_opcode (6,433,530x)
          .                               OpCode::Move => {
 10,455,458 ( 0.21%)                          let rb=(base + get_arg_b(i)) as usize;
  4,021,330 ( 0.08%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_b (804,266x)
 20,910,916 ( 0.42%)                          self.stack[ra as usize]=self.stack[rb].clone();
 41,017,566 ( 0.82%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (804,266x)
 41,017,566 ( 0.82%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut (804,266x)
 11,259,936 ( 0.23%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (804,266x)
 19,310,256 ( 0.39%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (804,266x)
    804,266 ( 0.02%)                      },
          .                               OpCode::LoadK => {
     72,546 ( 0.00%)                          let kid = get_arg_bx(i);
     60,455 ( 0.00%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_bx (12,091x)
    108,819 ( 0.00%)                          let kname = self.get_lua_constant(cl.get_proto_id(), kid as usize);
  1,849,971 ( 0.04%)  => /home/jice/lua-rs/src/state.rs:lua_rs::state::LuaState::get_lua_constant (12,091x)
    145,092 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::Closure::get_proto_id (12,091x)
    217,638 ( 0.00%)                          self.stack[ra as usize] = kname.clone();
    616,641 ( 0.01%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut (12,091x)
    169,274 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (12,091x)
    290,232 ( 0.01%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (12,091x)
     48,364 ( 0.00%)                      }
    169,327 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (12,091x)
          .                               OpCode::LoadBool => {
          .                                   let b=get_arg_b(i);
          .                                   self.stack[ra as usize]=TValue::Boolean(b!=0);
          .                                   let c=get_arg_c(i);
          .                                   if c != 0 {
          .                                       pc+=1; // skip next instruction (if C)
          .                                       unsafe { pcptr = pcptr.add(1);}
          .                                   }
          .                               },
          .                               OpCode::LoadNil => todo!(),
          .                               OpCode::GetUpVal => {
  2,400,240 ( 0.05%)                          let b=get_arg_b(i);
  2,000,200 ( 0.04%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_b (400,040x)
  9,200,920 ( 0.18%)                          self.stack[ra as usize] = cl.get_lua_upvalue(b as usize);
 20,402,040 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut (400,040x)
  5,600,560 ( 0.11%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (400,040x)
 58,005,800 ( 1.16%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::Closure::get_lua_upvalue (400,040x)
    400,040 ( 0.01%)                      },
          .                               OpCode::GetGlobal => {
         18 ( 0.00%)                          let kid = get_arg_bx(i);
         15 ( 0.00%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_bx (3x)
         27 ( 0.00%)                          let kname = self.get_lua_constant(cl.get_proto_id(), kid as usize);
        603 ( 0.00%)  => /home/jice/lua-rs/src/state.rs:lua_rs::state::LuaState::get_lua_constant (3x)
         36 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::Closure::get_proto_id (3x)
          6 ( 0.00%)                          self.saved_pc = pc;
         48 ( 0.00%)                          Self::get_tablev2(&mut self.stack, cl.get_envvalue(), &kname, Some(ra as usize));
      6,074 ( 0.00%)  => /home/jice/lua-rs/src/state.rs:lua_rs::state::LuaState::get_tablev2 (3x)
         30 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::Closure::get_envvalue (3x)
          6 ( 0.00%)                          base = self.base as u32;
         12 ( 0.00%)                      }
        201 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (3x)
          .                               OpCode::GetTable => {
    800,404 ( 0.02%)                          self.saved_pc = pc;
  5,202,626 ( 0.10%)                          let tableid = (base + get_arg_b(i)) as usize;
  2,001,010 ( 0.04%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_b (400,202x)
  1,600,808 ( 0.03%)                          let c=get_arg_c(i);
  2,001,010 ( 0.04%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_c (400,202x)
  2,401,210 ( 0.05%)                          let key = if RK_IS_K(c) {
  2,801,414 ( 0.06%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::RK_IS_K (400,202x)
         20 ( 0.00%)                              self.get_lua_constant(cl.get_proto_id(),(c & !BIT_RK) as usize)
        402 ( 0.00%)  => /home/jice/lua-rs/src/state.rs:lua_rs::state::LuaState::get_lua_constant (2x)
         24 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::Closure::get_proto_id (2x)
          .                                   } else {
  7,203,600 ( 0.14%)                              self.stack[(base + c) as usize].clone()
 20,410,200 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (400,200x)
  9,604,800 ( 0.19%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (400,200x)
          .                                   };
  3,601,818 ( 0.07%)                          Self::get_tablev(&mut self.stack, tableid, &key, Some(ra as usize));
234,121,258 ( 4.68%)  => /home/jice/lua-rs/src/state.rs:lua_rs::state::LuaState::get_tablev (400,202x)
    800,404 ( 0.02%)                          base = self.base as u32;
  1,600,808 ( 0.03%)                      },
  5,602,934 ( 0.11%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (400,202x)
          .                               OpCode::SetGlobal => {
          .                                   let g= cl.get_env().clone();
          .                                   let kid = get_arg_bx(i) as usize;
          .                                   let key = self.get_lua_constant(cl.get_proto_id(),kid);
          .                                   self.saved_pc = pc;
          .                                   let value=self.stack[ra as usize].clone();
          .                                   self.set_tablev(&TValue::Table(g), key, value);
          .                                   base = self.base as  u32;
          .                               },
          .                               OpCode::SetupVal => todo!(),
          .                               OpCode::SetTable => {
      8,200 ( 0.00%)                          self.saved_pc = pc;
     16,400 ( 0.00%)                          let b=get_arg_b(i);
     20,500 ( 0.00%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_b (4,100x)
     16,400 ( 0.00%)                          let c = get_arg_c(i);
     20,500 ( 0.00%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_c (4,100x)
     28,700 ( 0.00%)                          let key = if RK_IS_K(b) {
     28,700 ( 0.00%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::RK_IS_K (4,100x)
          .                                       self.get_lua_constant(cl.get_proto_id(),(b &!BIT_RK) as usize)
          .                                   } else {
     73,800 ( 0.00%)                              self.stack[(base + b) as usize].clone()
    209,100 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (4,100x)
     98,400 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (4,100x)
          .                                   };
     28,500 ( 0.00%)                          let value = if RK_IS_K(c) {
     28,700 ( 0.00%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::RK_IS_K (4,100x)
      1,000 ( 0.00%)                              self.get_lua_constant(cl.get_proto_id(),(c &!BIT_RK) as usize)
     15,300 ( 0.00%)  => /home/jice/lua-rs/src/state.rs:lua_rs::state::LuaState::get_lua_constant (100x)
      1,200 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::Closure::get_proto_id (100x)
          .                                   } else {
     72,000 ( 0.00%)                              self.stack[(base + c) as usize].clone()
    204,000 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (4,000x)
     96,000 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (4,000x)
          .                                   };
     73,800 ( 0.00%)                          self.set_tablev(&self.stack[ra as usize], key, value);
    209,100 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (4,100x)
  1,548,449 ( 0.03%)  => /home/jice/lua-rs/src/state.rs:lua_rs::state::LuaState::set_tablev (4,100x)
      8,200 ( 0.00%)                          base=self.base as u32;
     12,300 ( 0.00%)                      },
          3 ( 0.00%)                      OpCode::NewTable => {
         66 ( 0.00%)                          self.stack[ra as usize] = TValue::new_table();
        153 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut (3x)
         42 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (3x)
      1,854 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::new_table (3x)
          6 ( 0.00%)                          self.saved_pc = pc;
          6 ( 0.00%)                          base=  self.base as u32;
          .                               },
          .                               OpCode::OpSelf => todo!(),
132,022,000 ( 2.64%)                      OpCode::Add => arith_op!(+,OpCode::Add,cl,self,i,base,ra,pc),
122,420,400 ( 2.45%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (2,400,400x)
 61,210,200 ( 1.22%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut (1,200,200x)
 50,408,400 ( 1.01%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (3,600,600x)
 57,609,600 ( 1.15%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (2,400,400x)
 33,605,600 ( 0.67%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::get_number_value (2,400,400x)
 24,004,000 ( 0.48%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::is_number (2,400,400x)
 16,802,800 ( 0.34%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::RK_IS_K (2,400,400x)
  6,001,000 ( 0.12%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_b (1,200,200x)
  6,001,000 ( 0.12%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_c (1,200,200x)
 80,800,000 ( 1.62%)                      OpCode::Sub => arith_op!(-,OpCode::Sub,cl,self,i,base,ra,pc),
 40,800,000 ( 0.82%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (800,000x)
 40,800,000 ( 0.82%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut (800,000x)
122,400,000 ( 2.45%)  => /home/jice/lua-rs/src/state.rs:lua_rs::state::LuaState::get_lua_constant (800,000x)
 33,600,000 ( 0.67%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (2,400,000x)
 19,200,000 ( 0.38%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (800,000x)
 22,400,000 ( 0.45%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::get_number_value (1,600,000x)
 16,000,000 ( 0.32%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::is_number (1,600,000x)
 11,200,000 ( 0.22%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::RK_IS_K (1,600,000x)
  4,000,000 ( 0.08%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_b (800,000x)
  4,000,000 ( 0.08%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_c (800,000x)
  9,600,000 ( 0.19%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::Closure::get_proto_id (800,000x)
128,422,000 ( 2.57%)                      OpCode::Mul => arith_op!(*,OpCode::Mul,cl,self,i,base,ra,pc),
102,020,400 ( 2.04%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (2,000,400x)
 61,210,200 ( 1.22%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut (1,200,200x)
 61,200,000 ( 1.22%)  => /home/jice/lua-rs/src/state.rs:lua_rs::state::LuaState::get_lua_constant (400,000x)
 50,408,400 ( 1.01%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (3,600,600x)
 48,009,600 ( 0.96%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (2,000,400x)
 33,605,600 ( 0.67%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::get_number_value (2,400,400x)
 24,004,000 ( 0.48%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::is_number (2,400,400x)
 16,802,800 ( 0.34%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::RK_IS_K (2,400,400x)
  6,001,000 ( 0.12%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_b (1,200,200x)
  6,001,000 ( 0.12%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_c (1,200,200x)
  4,800,000 ( 0.10%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::Closure::get_proto_id (400,000x)
 40,400,110 ( 0.81%)                      OpCode::Div => arith_op!(/,OpCode::Div,cl,self,i,base,ra,pc),
 20,400,102 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (400,002x)
 20,400,051 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut (400,001x)
 61,200,000 ( 1.22%)  => /home/jice/lua-rs/src/state.rs:lua_rs::state::LuaState::get_lua_constant (400,000x)
 16,800,042 ( 0.34%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (1,200,003x)
  9,600,048 ( 0.19%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (400,002x)
 11,200,028 ( 0.22%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::get_number_value (800,002x)
  8,000,020 ( 0.16%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::is_number (800,002x)
  5,600,014 ( 0.11%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::RK_IS_K (800,002x)
  2,000,005 ( 0.04%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_b (400,001x)
  2,000,005 ( 0.04%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_c (400,001x)
  4,800,000 ( 0.10%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::Closure::get_proto_id (400,000x)
          .                               OpCode::Mod => arith_op!(%,OpCode::Mod,cl,self,i,base,ra,pc),
          .                               OpCode::Pow => todo!(),
          .                               OpCode::UnaryMinus => todo!(),
          .                               OpCode::Not => todo!(),
          .                               OpCode::Len => todo!(),
          .                               OpCode::Concat => todo!(),
          .                               OpCode::Jmp => todo!(),
          .                               OpCode::Eq => todo!(),
-- line 170 ----------------------------------------
-- line 178 ----------------------------------------
          .                                       pc = (pc as i32 + jump) as usize;
          .                                       unsafe { pcptr = pcptr.offset(jump as isize);}
          .                                   }
          .                                   pc += 1;
          .                                   unsafe{ pcptr = pcptr.add(1);}
          .                               }
          .                               OpCode::TestSet => todo!(),
          .                               OpCode::Call => {
  1,600,252 ( 0.03%)                          let b = get_arg_b(i);
  2,000,315 ( 0.04%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_b (400,063x)
  3,600,567 ( 0.07%)                          let nresults = get_arg_c(i) as i32 - 1;
  2,000,315 ( 0.04%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_c (400,063x)
  1,200,187 ( 0.02%)                          if b != 0 {
  6,000,915 ( 0.12%)                              self.stack.resize((ra + b) as usize, TValue::Nil); // top = ra+b
 36,405,605 ( 0.73%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::resize (400,061x)
          .                                   } // else previous instruction set top
    800,126 ( 0.02%)                          self.saved_pc = pc;
  8,001,251 ( 0.16%)                          match self.dprecall(ra as usize, nresults as i32) {
404,138,513 ( 8.09%)  => /home/jice/lua-rs/src/ldo.rs:lua_rs::ldo::<impl lua_rs::state::LuaState>::dprecall (400,063x)
          .                                       Ok(PrecallStatus::Lua) => {
  3,200,480 ( 0.06%)                                  nexec_calls += 1;
          .                                           // restart luaV_execute over new Lua function
          .                                           continue 'reentry;
          .                                       }
          .                                       Ok(PrecallStatus::Rust) => {
          .                                           // it was a Rust function (`precall' called it); adjust results
          9 ( 0.00%)                                  if nresults > 0 {
          .                                               self.stack.resize(self.base_ci[self.ci].top, TValue::Nil);
          .                                           }
          6 ( 0.00%)                                  base = self.base as u32;
          .                                       }
          .                                       Ok(PrecallStatus::RustYield) => {
          .                                           return Ok(()); // yield
          .                                       }
          .                                       Err(e) => {
          .                                           return Err(e);
          .                                       }
          .                                   }
          3 ( 0.00%)                      }
          .                               OpCode::TailCall => todo!(),
          .                               OpCode::Return => {
  1,600,244 ( 0.03%)                          let b = get_arg_b(i);
  2,000,305 ( 0.04%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_b (400,061x)
    800,122 ( 0.02%)                          if b != 0 {
  9,201,403 ( 0.18%)                              self.stack.resize((ra + b - 1) as usize, TValue::Nil);
 36,446,438 ( 0.73%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::resize (400,061x)
          .                                   }
  3,200,488 ( 0.06%)                          if !self.open_upval.is_empty() {
  2,400,366 ( 0.05%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::is_empty (400,061x)
  2,400,366 ( 0.05%)                              self.close_func(base as StkId);
 24,405,202 ( 0.49%)  => /home/jice/lua-rs/src/state.rs:lua_rs::state::LuaState::close_func (400,061x)
          .                                   }
    800,122 ( 0.02%)                          self.saved_pc = pc;
  2,000,305 ( 0.04%)                          let b = self.poscall(ra);
283,625,205 ( 5.68%)  => /home/jice/lua-rs/src/state.rs:lua_rs::state::LuaState::poscall (400,061x)
  2,800,427 ( 0.06%)                          nexec_calls -= 1;
  1,200,182 ( 0.02%)                          if nexec_calls == 0 {
          1 ( 0.00%)                              return Ok(());
          .                                   }
    800,120 ( 0.02%)                          if b {
  7,201,080 ( 0.14%)                              self.stack.resize(self.base_ci[self.ci].top, TValue::Nil);
 20,403,060 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (400,060x)
122,433,900 ( 2.45%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::resize (400,060x)
          .                                   }
          .                                   continue 'reentry;
          .                               }
          .                               OpCode::ForLoop => {
  7,756,807 ( 0.16%)                          let step = self.stack[ra as usize+2].get_number_value();
 20,820,903 ( 0.42%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (408,253x)
  5,715,542 ( 0.11%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::get_number_value (408,253x)
  6,123,795 ( 0.12%)                          let idx = self.stack[ra as usize].get_number_value() + step;
 20,820,903 ( 0.42%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (408,253x)
  5,715,542 ( 0.11%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::get_number_value (408,253x)
  7,348,554 ( 0.15%)                          let limit = self.stack[ra as usize+1].get_number_value();
 20,820,903 ( 0.42%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (408,253x)
  5,715,542 ( 0.11%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::get_number_value (408,253x)
  1,224,759 ( 0.02%)                          let end_loop = if step > 0.0 {
  1,633,012 ( 0.03%)                              idx <= limit
          .                                   } else {
          .                                       limit <= idx
          .                                   };
  1,220,716 ( 0.02%)                          if end_loop {
          .                                       // jump back
  1,616,840 ( 0.03%)                              let jump = get_arg_sbx(i);
  6,871,570 ( 0.14%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_sbx (404,210x)
  3,233,680 ( 0.06%)                              pc = (pc as i32 + jump) as usize;
  2,021,050 ( 0.04%)                              unsafe { pcptr = pcptr.offset(jump as isize);}
  8,084,200 ( 0.16%)                              self.stack[ra as usize] = TValue::Number(idx); // update internal index
 20,614,710 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut (404,210x)
  5,658,940 ( 0.11%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (404,210x)
 10,509,460 ( 0.21%)                              self.stack[ra as usize+3] = TValue::Number(idx); // ...and external index
 20,614,710 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut (404,210x)
  5,658,940 ( 0.11%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (404,210x)
          .                                   }
          .                               },
          .                               OpCode::ForPrep => {
      8,086 ( 0.00%)                          self.saved_pc = pc;
     60,645 ( 0.00%)                          if ! Self::to_number(&mut self.stack, ra as usize,ra as usize) {
    125,333 ( 0.00%)  => /home/jice/lua-rs/src/state.rs:lua_rs::state::LuaState::to_number (4,043x)
     72,774 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::deref::DerefMut>::deref_mut (4,043x)
          .                                       return self.run_error("'for' initial value must be a number");
          .                                   }
    101,075 ( 0.00%)                          if ! Self::to_number(&mut self.stack,ra as usize+1, ra as usize+1) {
    125,333 ( 0.00%)  => /home/jice/lua-rs/src/state.rs:lua_rs::state::LuaState::to_number (4,043x)
     72,774 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::deref::DerefMut>::deref_mut (4,043x)
          .                                       return self.run_error("'for' limit must be a number");
          .                                   }
    101,075 ( 0.00%)                          if ! Self::to_number(&mut self.stack,ra as usize+2, ra as usize+2) {
    125,333 ( 0.00%)  => /home/jice/lua-rs/src/state.rs:lua_rs::state::LuaState::to_number (4,043x)
     72,774 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::deref::DerefMut>::deref_mut (4,043x)
          .                                       return self.run_error("'for' step must be a number");
          .                                   }
          .                                   // init = init - step
     68,731 ( 0.00%)                          self.stack[ra as usize] = TValue::Number(
    206,193 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut (4,043x)
     56,602 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (4,043x)
     52,559 ( 0.00%)                              self.stack[ra as usize].get_number_value()
    206,193 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (4,043x)
     56,602 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::get_number_value (4,043x)
     68,731 ( 0.00%)                              - self.stack[ra as usize+2].get_number_value()
    206,193 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (4,043x)
     56,602 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::get_number_value (4,043x)
      4,043 ( 0.00%)                          );
     16,172 ( 0.00%)                          let jump = get_arg_sbx(i);
     68,731 ( 0.00%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_sbx (4,043x)
     32,344 ( 0.00%)                          pc = (pc as i32 + jump) as usize;
     20,215 ( 0.00%)                          unsafe { pcptr = pcptr.offset(jump as isize);}
      4,043 ( 0.00%)                      },
          .                               OpCode::TForLoop => todo!(),
          .                               OpCode::SetList => todo!(),
          .                               OpCode::Close => todo!(),
          .                               OpCode::Closure => {
         52 ( 0.00%)                          let pci = unsafe { *pcptr};
         20 ( 0.00%)                          if let Closure::Lua(cl) = &*cl {
         16 ( 0.00%)                              let pid = get_arg_bx(i);
         20 ( 0.00%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_bx (4x)
         72 ( 0.00%)                              let pid = self.protos[cl.proto].p[pid as usize];
        408 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (8x)
         28 ( 0.00%)                              let p = &self.protos[pid];
        204 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (4x)
         12 ( 0.00%)                              let nup = p.nups;
         40 ( 0.00%)                              let mut ncl = LClosure::new(pid, cl.env.clone());
        192 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::clone::Clone>::clone (4x)
        380 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::LClosure::new (4x)
        124 ( 0.00%)                              for _ in 0..nup {
        316 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/range.rs:core::iter::range::<impl core::iter::traits::iterator::Iterator for core::ops::range::Range<A>>::next (8x)
         20 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/traits/collect.rs:<I as core::iter::traits::collect::IntoIterator>::into_iter (4x)
         48 ( 0.00%)                                  if get_opcode(pci) == OpCode::GetUpVal {
        156 ( 0.00%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_opcode (4x)
         44 ( 0.00%)  => /home/jice/lua-rs/src/opcodes.rs:<lua_rs::opcodes::unformatted::OpCode as core::cmp::PartialEq>::eq (4x)
          .                                               let upvalid = get_arg_b(pci);
          .                                               ncl.upvalues.push(cl.upvalues[upvalid as usize].clone());
          .                                           } else {
         64 ( 0.00%)                                      debug_assert!(get_opcode(pci) == OpCode::Move);
        156 ( 0.00%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_opcode (4x)
         44 ( 0.00%)  => /home/jice/lua-rs/src/opcodes.rs:<lua_rs::opcodes::unformatted::OpCode as core::cmp::PartialEq>::eq (4x)
         16 ( 0.00%)                                      let b = get_arg_b(pci);
         20 ( 0.00%)  => /home/jice/lua-rs/src/opcodes.rs:lua_rs::opcodes::get_arg_b (4x)
         92 ( 0.00%)                                      ncl.upvalues.push(Self::find_upval(&mut self.open_upval, &mut self.stack, base + b));
         72 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::deref::DerefMut>::deref_mut (4x)
      2,825 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::push (4x)
      2,788 ( 0.00%)  => /home/jice/lua-rs/src/state.rs:lua_rs::state::LuaState::find_upval (4x)
          .                                           }
          .                                       }
        172 ( 0.00%)                              self.stack[ra as usize] = TValue::Function(Rc::new(Closure::Lua(ncl)));
        204 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut (4x)
         56 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (4x)
      1,531 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:alloc::rc::Rc<T>::new (4x)
          8 ( 0.00%)                              self.saved_pc = pc;
          8 ( 0.00%)                              base = self.base as u32;
          4 ( 0.00%)                          } else {
          .                                       unreachable!()
          .                                   }
          4 ( 0.00%)                      }
          .                               OpCode::VarArg => todo!(),
          .                           }
          .                       }
  2,400,362 ( 0.05%)          }
 39,206,351 ( 0.78%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<alloc::rc::Rc<lua_rs::object::Closure>> (800,121x)
          3 ( 0.00%)      }
          .           }
          .           
          .           #[cfg(feature="debug_logs")]
          .           fn dump_function_header(state:&LuaState, cl: &LClosure) {
          .               let nup = cl.upvalues.len();
          .               let proto = &state.protos[cl.proto];
          .               let nk = proto.k.len();
          .               if proto.linedefined == proto.lastlinedefined {
-- line 311 ----------------------------------------

255,272,370 ( 5.11%)  <counts for unidentified lines in src/vm.rs>

--------------------------------------------------------------------------------
-- Auto-annotated source: src/state.rs
--------------------------------------------------------------------------------
Ir                  

-- line 14 ----------------------------------------
         .               LUA_REGISTRYINDEX,
         .           };
         .           
         .           pub type PanicFunction = fn(&mut LuaState) -> i32;
         .           
         .           pub const EXTRA_STACK: usize = 5;
         .           
         .           /// informations about a call
 4,400,770 ( 0.09%)  #[derive(Default)]
         .           pub struct CallInfo {
         .               /// base for this function
   800,140 ( 0.02%)      pub base: StkId,
   800,140 ( 0.02%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<usize as core::default::Default>::default (400,070x)
         .               /// function index in the stack
   800,140 ( 0.02%)      pub func: StkId,
   800,140 ( 0.02%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<usize as core::default::Default>::default (400,070x)
         .               /// top for this function
   800,140 ( 0.02%)      pub top: StkId,
   800,140 ( 0.02%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<usize as core::default::Default>::default (400,070x)
         .               /// program counter
   800,140 ( 0.02%)      pub saved_pc: InstId,
   800,140 ( 0.02%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<usize as core::default::Default>::default (400,070x)
         .               /// expected number of results from this function
   800,140 ( 0.02%)      pub nresults: i32,
   800,140 ( 0.02%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<i32 as core::default::Default>::default (400,070x)
         .               /// number of tail calls lost under this entry
 3,600,630 ( 0.07%)      pub tailcalls: usize,
   800,140 ( 0.02%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<usize as core::default::Default>::default (400,070x)
         .           }
         .           
         .           impl CallInfo {
         3 ( 0.00%)      pub(crate) fn new() -> Self {
         2 ( 0.00%)          Self::default()
        42 ( 0.00%)  => src/state.rs:<lua_rs::state::CallInfo as core::default::Default>::default (1x)
         2 ( 0.00%)      }
         .           }
         .           
         .           pub struct GlobalState {
         .               /// to be called in unprotected errors
         .               pub panic: Option<PanicFunction>,
         .               /// metatables for basic types
         .               pub mt: Vec<Option<TableRef>>,
         .               pub registry: TValue,
         .           }
         .           
         .           impl Default for GlobalState {
         3 ( 0.00%)      fn default() -> Self {
         5 ( 0.00%)          let mut mt = Vec::new();
         7 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T>::new (1x)
       111 ( 0.00%)          for _ in 0..TVALUE_TYPE_COUNT {
       487 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/range.rs:core::iter::range::<impl core::iter::traits::iterator::Iterator for core::ops::range::Range<A>>::next (10x)
         5 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/traits/collect.rs:<I as core::iter::traits::collect::IntoIterator>::into_iter (1x)
        63 ( 0.00%)              mt.push(None)
     2,985 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::push (9x)
         .                   }
        12 ( 0.00%)          Self {
         1 ( 0.00%)              panic: None,
         5 ( 0.00%)              mt,
         4 ( 0.00%)              registry: TValue::new_table(),
       776 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::new_table (1x)
         .                   }
         3 ( 0.00%)      }
         .           }
         .           
        61 ( 0.00%)  #[derive(Default)]
        13 ( 0.00%)  => ???:0x000000000011a060 (1x)
         .           pub struct LuaState {
         3 ( 0.00%)      pub g: GlobalState,
     4,491 ( 0.00%)  => src/state.rs:<lua_rs::state::GlobalState as core::default::Default>::default (1x)
         .               /// base of current function
         3 ( 0.00%)      pub base: StkId,
         2 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<usize as core::default::Default>::default (1x)
         .               /// `savedpc' of current function
         3 ( 0.00%)      pub saved_pc: InstId,
         2 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<usize as core::default::Default>::default (1x)
         .               /// stack base
         4 ( 0.00%)      pub stack: Vec<TValue>,
         7 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T> as core::default::Default>::default (1x)
         .               /// current error handling function (stack index)
         3 ( 0.00%)      pub errfunc: StkId,
         2 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<usize as core::default::Default>::default (1x)
         .               /// number of nested Rust calls
         3 ( 0.00%)      pub n_rcalls: usize,
         2 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<usize as core::default::Default>::default (1x)
         .               /// call info for current function
         3 ( 0.00%)      pub ci: CallId,
         2 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<usize as core::default::Default>::default (1x)
         .               /// list of nested CallInfo
         4 ( 0.00%)      pub base_ci: Vec<CallInfo>,
         7 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T> as core::default::Default>::default (1x)
         3 ( 0.00%)      pub allowhook: bool,
         4 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<bool as core::default::Default>::default (1x)
         3 ( 0.00%)      pub hookmask: usize,
         2 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs:<usize as core::default::Default>::default (1x)
         .               /// table of globals
         6 ( 0.00%)      pub l_gt: TableRef,
       766 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::default::Default>::default (1x)
         2 ( 0.00%)      pub gtvalue: TValue,
         3 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::default::Default>::default (1x)
         .               /// temporary place for environments
         6 ( 0.00%)      pub env: TableRef,
       766 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::default::Default>::default (1x)
         2 ( 0.00%)      pub envvalue: TValue,
         3 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::default::Default>::default (1x)
         .               /// list of open upvalues
         4 ( 0.00%)      pub open_upval: Vec<UpVal>,
         7 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T> as core::default::Default>::default (1x)
         .               /// all closures prototypes
         4 ( 0.00%)      pub protos: Vec<Proto>,
         7 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T> as core::default::Default>::default (1x)
         .           }
         .           
         .           impl LuaState {
         3 ( 0.00%)      pub(crate) fn init_stack(&mut self) {
         .                   // initialize first ci
         3 ( 0.00%)          let mut ci = CallInfo::new();
        49 ( 0.00%)  => src/state.rs:lua_rs::state::CallInfo::new (1x)
         .                   // `function' entry for this `ci'
         .                   //self.stack.push(TValue::Nil);
         1 ( 0.00%)          self.base = 0;
         1 ( 0.00%)          ci.base = 0;
         7 ( 0.00%)          ci.top = 1 + LUA_MINSTACK;
         9 ( 0.00%)          self.base_ci.push(ci);
       887 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::push (1x)
        13 ( 0.00%)  => ???:0x000000000011a060 (1x)
         2 ( 0.00%)      }
         .               #[inline]
20,958,548 ( 0.42%)      pub fn get_lua_constant(&self, protoid: usize, kid : usize) -> TValue {
19,346,352 ( 0.39%)          return self.protos[protoid].k[kid].clone();
164,443,992 ( 3.29%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (3,224,392x)
38,692,992 ( 0.77%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (1,612,196x)
 3,224,392 ( 0.06%)      }    
        18 ( 0.00%)      pub(crate) fn push_rust_function(&mut self, func: LuaRustFunction) {
        18 ( 0.00%)          self.push_rust_closure(func, 0);
     5,765 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::push_rust_closure (6x)
        12 ( 0.00%)      }
        80 ( 0.00%)      pub(crate) fn push_string(&mut self, value: &str) {
       220 ( 0.00%)          self.stack.push(TValue::String(Rc::new(value.to_owned())));
     8,512 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/str.rs:alloc::str::<impl alloc::borrow::ToOwned for str>::to_owned (20x)
     5,394 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:alloc::rc::Rc<T>::new (20x)
     1,994 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::push (20x)
        40 ( 0.00%)      }
         9 ( 0.00%)      pub(crate) fn push_number(&mut self, value: LuaNumber) {
        15 ( 0.00%)          self.stack.push(TValue::Number(value));
       135 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::push (3x)
         6 ( 0.00%)      }
         .               pub(crate) fn push_nil(&mut self) {
         .                   self.stack.push(TValue::Nil);
         .               }
        35 ( 0.00%)      pub(crate) fn call(&mut self, nargs: usize, nresults: i32) -> Result<(), LuaError> {
        45 ( 0.00%)          self.api_check_nelems(nargs + 1);
       165 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::api_check_nelems (5x)
        10 ( 0.00%)          self.check_results(nargs, nresults);
       540 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::check_results (5x)
        20 ( 0.00%)          let len = self.stack.len();
        15 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (5x)
       130 ( 0.00%)          self.dcall(len - nargs - 1, nresults)?;
   734,299 ( 0.01%)  => /home/jice/lua-rs/src/ldo.rs:lua_rs::ldo::<impl lua_rs::state::LuaState>::dcall (4x)
     2,080 ( 0.00%)  => /home/jice/lua-rs/src/ldo.rs:lua_rs::ldo::<impl lua_rs::state::LuaState>::dcall'2 (1x)
        60 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/result.rs:<core::result::Result<T,E> as core::ops::try_trait::Try>::branch (5x)
         5 ( 0.00%)          self.adjust_results(nresults);
        60 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::adjust_results (5x)
         5 ( 0.00%)          Ok(())
        20 ( 0.00%)      }
         .           
         .               #[inline]
       480 ( 0.00%)      fn api_check_nelems(&self, n: usize) {
     1,728 ( 0.00%)          debug_assert!(n as i32 <= self.stack.len() as i32 - self.base as i32);
       288 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (96x)
       192 ( 0.00%)      }
         .               #[inline]
        42 ( 0.00%)      pub(crate) fn check_results(&self, nargs: usize, nresults: i32) {
        48 ( 0.00%)          debug_assert!(
        28 ( 0.00%)              nresults == LUA_MULTRET
       115 ( 0.00%)                  || self.base_ci.last().unwrap().top as isize - self.stack.len() as isize
        90 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::deref::Deref>::deref (5x)
        75 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/slice/mod.rs:core::slice::<impl [T]>::last (5x)
        70 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/option.rs:core::option::Option<T>::unwrap (5x)
        15 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (5x)
        35 ( 0.00%)                      >= nresults as isize - nargs as isize
         .                   );
        12 ( 0.00%)      }
         .           
       910 ( 0.00%)      pub(crate) fn push_rust_closure(&mut self, func: LuaRustFunction, nup_values: usize) {
       273 ( 0.00%)          self.api_check_nelems(nup_values);
     3,003 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::api_check_nelems (91x)
       182 ( 0.00%)          let env = self.get_current_env();
    18,099 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::get_current_env (91x)
       910 ( 0.00%)          let mut cl = RClosure::new(func, Rc::clone(&env));
     8,645 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::RClosure::new (91x)
     4,368 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::clone::Clone>::clone (91x)
     1,941 ( 0.00%)          for _ in 0..nup_values {
     2,701 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/range.rs:core::iter::range::<impl core::iter::traits::iterator::Iterator for core::ops::range::Range<A>>::next (94x)
       455 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/traits/collect.rs:<I as core::iter::traits::collect::IntoIterator>::into_iter (91x)
        54 ( 0.00%)              cl.upvalues.push(self.stack.pop().unwrap());
     2,448 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::push (3x)
       108 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::pop (3x)
        60 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/option.rs:core::option::Option<T>::unwrap (3x)
         .                   }
       546 ( 0.00%)          self.stack
     4,914 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::push (91x)
     2,275 ( 0.00%)              .push(TValue::Function(Rc::new(Closure::Rust(cl))));
    35,220 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:alloc::rc::Rc<T>::new (91x)
       455 ( 0.00%)      }
     4,459 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<alloc::rc::Rc<core::cell::RefCell<lua_rs::table::Table>>> (91x)
         .           
       273 ( 0.00%)      fn get_current_env(&self) -> TableRef {
       364 ( 0.00%)          if self.base_ci.len() == 1 {
       273 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (91x)
         .                       // no enclosing function
         .                       // use global table as environment
        16 ( 0.00%)              return Rc::clone(&self.l_gt);
       192 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::clone::Clone>::clone (4x)
         .                   } else {
       957 ( 0.00%)              let ci_stkid = self.base_ci.last().unwrap().func;
     1,566 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::deref::Deref>::deref (87x)
     1,305 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/slice/mod.rs:core::slice::<impl [T]>::last (87x)
     1,218 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/option.rs:core::option::Option<T>::unwrap (87x)
       957 ( 0.00%)              if let TValue::Function(cl) = &self.stack[ci_stkid] {
     4,437 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (87x)
       261 ( 0.00%)                  return cl.get_env();
     5,481 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::Closure::get_env (87x)
         .                       }
         .                   }
         .                   unreachable!()
       273 ( 0.00%)      }
         .           
         .               pub(crate) fn run_error(&mut self, msg: &str) -> Result<(), LuaError> {
         .                   let fullmsg = {
         .                       let ci = &self.base_ci[self.ci];
         .                       let luacl = &self.stack[ci.func];
         .                       let luacl = luacl.get_lua_closure();
         .                       let pc = self.saved_pc;
         .                       let proto = &self.protos[luacl.proto];
-- line 177 ----------------------------------------
-- line 178 ----------------------------------------
         .                       let line = proto.lineinfo[pc];
         .                       let chunk_id = &proto.source;
         .                       format!("{}:{} {}", chunk_id, line, msg)
         .                   };
         .                   self.stack.push(TValue::new_string(&fullmsg));
         .                   Err(LuaError::RuntimeError)
         .               }
         .           
        24 ( 0.00%)      pub(crate) fn adjust_results(&mut self, nresults: i32) {
        50 ( 0.00%)          if nresults == LUA_MULTRET && self.stack.len() >= self.base_ci[self.ci].top {
        51 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (1x)
         3 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (1x)
         .                       self.base_ci[self.ci].top = self.stack.len();
         .                   }
        12 ( 0.00%)      }
         .           
        75 ( 0.00%)      pub(crate) fn push_value(&mut self, index: isize) {
       250 ( 0.00%)          self.stack.push(self.index2adr(index).clone());
     2,190 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::index2adr (25x)
     1,800 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (25x)
     1,125 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::push (25x)
        50 ( 0.00%)      }
         .           
         .               /// create a global variable `key` with last value on stack
        18 ( 0.00%)      pub(crate) fn set_global(&mut self, key: &str) {
         6 ( 0.00%)          self.set_field(LUA_GLOBALSINDEX, key);
    10,511 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::set_field (3x)
         6 ( 0.00%)      }
         .           
         8 ( 0.00%)      pub(crate) fn push_literal(&mut self, value: &str) {
        14 ( 0.00%)          self.stack.push(TValue::new_string(value));
     1,871 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::new_string (2x)
        90 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::push (2x)
         4 ( 0.00%)      }
         .           
        12 ( 0.00%)      pub(crate) fn create_table(&mut self) {
        42 ( 0.00%)          self.stack.push(TValue::new_table());
     4,442 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::new_table (6x)
       270 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::push (6x)
        12 ( 0.00%)      }
         .           
         5 ( 0.00%)      pub(crate) fn set_metatable(&mut self, objindex: isize) {
        19 ( 0.00%)          debug_assert!(self.stack.len() >= 1);
         3 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (1x)
        13 ( 0.00%)          let mt = self.stack.pop().unwrap();
        36 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::pop (1x)
        20 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/option.rs:core::option::Option<T>::unwrap (1x)
        18 ( 0.00%)          let mt = if mt.is_nil() { None } else { Some(mt) };
         9 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::is_nil (1x)
         .                   let objtype = {
         9 ( 0.00%)              let objindex = if objindex < 0 && objindex > LUA_REGISTRYINDEX {
         6 ( 0.00%)                  objindex + 1
         .                       } else {
         .                           objindex
         .                       };
         5 ( 0.00%)              let obj = self.index2adr(objindex);
       125 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::index2adr (1x)
         4 ( 0.00%)              match obj {
         3 ( 0.00%)                  TValue::Table(rcobj) => {
        12 ( 0.00%)                      if let Some(TValue::Table(rcmt)) = mt {
        33 ( 0.00%)                          rcobj.borrow_mut().metatable = Some(rcmt.clone());
        84 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:core::cell::RefCell<T>::borrow_mut (1x)
        48 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::clone::Clone>::clone (1x)
        28 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<core::cell::RefMut<lua_rs::table::Table>> (1x)
        12 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<core::option::Option<alloc::rc::Rc<core::cell::RefCell<lua_rs::table::Table>>>> (1x)
         5 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:<core::cell::RefMut<T> as core::ops::deref::DerefMut>::deref_mut (1x)
         .                                   return;
         4 ( 0.00%)                      } else {
        49 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<alloc::rc::Rc<core::cell::RefCell<lua_rs::table::Table>>> (1x)
         .                                   rcobj.borrow_mut().metatable = None;
         .                                   return;
         .                               }
         .                           }
         .                           TValue::UserData(rcobj) => {
         .                               if let Some(TValue::Table(rcmt)) = mt {
         .                                   rcobj.borrow_mut().metatable = Some(rcmt.clone());
         .                                   return;
-- line 233 ----------------------------------------
-- line 239 ----------------------------------------
         .                           _ => obj.type_as_usize(),
         .                       }
         .                   };
         .                   if let Some(TValue::Table(rcmt)) = mt {
         .                       self.g.mt[objtype] = Some(rcmt.clone());
         .                   } else {
         .                       self.g.mt[objtype] = None;
         .                   }
        20 ( 0.00%)      }
         .           
    25,200 ( 0.00%)      pub(crate) fn set_tablev(&self, tvalue: &TValue, key: TValue, value: TValue) {
         .                   // TODO NEWINDEX metamethods
    46,200 ( 0.00%)          if let TValue::Table(rt) = tvalue {
   117,600 ( 0.00%)              rt.borrow_mut().set(key, value);
 1,224,896 ( 0.02%)  => /home/jice/lua-rs/src/table.rs:lua_rs::table::Table::set (4,200x)
   352,800 ( 0.01%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:core::cell::RefCell<T>::borrow_mut (4,200x)
   117,600 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<core::cell::RefMut<lua_rs::table::Table>> (4,200x)
    21,000 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:<core::cell::RefMut<T> as core::ops::deref::DerefMut>::deref_mut (4,200x)
         .                       return;
         .                   } else {
         .                       unreachable!()
         .                   }
     8,400 ( 0.00%)      }
         .           
        81 ( 0.00%)      pub(crate) fn get_tablev2(stack: &mut Vec<TValue>, t: &TValue, key: &TValue, val: Option<StkId>) {
         .                  // TODO INDEX metamethods
        72 ( 0.00%)         if let TValue::Table(rt) = t {
        27 ( 0.00%)          let mut rt = rt.clone();
       432 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::clone::Clone>::clone (9x)
         .                   loop {
         .                       let newrt;
         .                       {
        72 ( 0.00%)                  let mut rtmut = rt.borrow_mut();
       756 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:core::cell::RefCell<T>::borrow_mut (9x)
       153 ( 0.00%)                  match rtmut.get(key) {
    11,430 ( 0.00%)  => /home/jice/lua-rs/src/table.rs:lua_rs::table::Table::get (9x)
        45 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:<core::cell::RefMut<T> as core::ops::deref::DerefMut>::deref_mut (9x)
        12 ( 0.00%)                      Some(value) => {
         .                                   // found a value, put it on stack
         8 ( 0.00%)                          match val {
        63 ( 0.00%)                              Some(idx) => stack[idx] = value.clone(),
       216 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (3x)
       153 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut (3x)
        42 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (3x)
         7 ( 0.00%)                              None => return stack.push(value.clone()),
        72 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (1x)
        45 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::push (1x)
         .                                   }
         3 ( 0.00%)                          return;
         .                               }
         .                               None => {
        60 ( 0.00%)                          if let Some(ref mt) = rtmut.metatable {
        25 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:<core::cell::RefMut<T> as core::ops::deref::Deref>::deref (5x)
         .                                       // not found. try with the metatable
         .                                       newrt = mt.clone();
         .                                   } else {
         .                                       // no metatable, put Nil on stack
        10 ( 0.00%)                              match val {
         .                                           Some(idx) => stack[idx] = TValue::Nil,
        30 ( 0.00%)                                  None => stack.push(TValue::Nil),
       225 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::push (5x)
         .                                       }
         .                                       return;
         .                                   }
         .                               }
         .                           }
        36 ( 0.00%)              }
       252 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<core::cell::RefMut<lua_rs::table::Table>> (9x)
         .                       rt = newrt;
         9 ( 0.00%)          }
        27 ( 0.00%)      }
       441 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<alloc::rc::Rc<core::cell::RefCell<lua_rs::table::Table>>> (9x)
        18 ( 0.00%)      }
         .               /// put field value `key` from table `t` on stack
 3,201,616 ( 0.06%)      pub(crate) fn get_tablev(stack : &mut Vec<TValue>, tableid: usize, key: &TValue, val: Option<StkId>) {
         .           
         .                   // TODO INDEX metamethods
 4,402,222 ( 0.09%)          if let TValue::Table(rt) = &stack[tableid] {
20,410,302 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (400,202x)
 1,200,606 ( 0.02%)              let mut rt = rt.clone();
19,209,696 ( 0.38%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::clone::Clone>::clone (400,202x)
         .                       loop {
         .                           let newrt;
         .                           {
 3,201,616 ( 0.06%)                      let mut rtmut = rt.borrow_mut();
33,616,968 ( 0.67%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:core::cell::RefCell<T>::borrow_mut (400,202x)
 6,803,434 ( 0.14%)                      match rtmut.get(key) {
50,428,338 ( 1.01%)  => /home/jice/lua-rs/src/table.rs:lua_rs::table::Table::get (400,202x)
 2,001,010 ( 0.04%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:<core::cell::RefMut<T> as core::ops::deref::DerefMut>::deref_mut (400,202x)
 1,200,606 ( 0.02%)                          Some(value) => {
         .                                       // found a value, put it on stack
   800,404 ( 0.02%)                              match val {
 8,404,242 ( 0.17%)                                  Some(idx) => stack[idx] = value.clone(),
20,410,302 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut (400,202x)
 9,604,944 ( 0.19%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (400,202x)
 5,602,934 ( 0.11%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (400,202x)
         .                                           None => return stack.push(value.clone()),
         .                                       }
   400,202 ( 0.01%)                              return;
         .                                   }
         .                                   None => {
         .                                       if let Some(ref mt) = rtmut.metatable {
         .                                           // not found. try with the metatable
         .                                           newrt = mt.clone();
         .                                       } else {
         .                                           // no metatable, put Nil on stack
         .                                           match val {
         .                                               Some(idx) => stack[idx] = TValue::Nil,
         .                                               None => stack.push(TValue::Nil),
         .                                           }
         .                                           return;
         .                                       }
         .                                   }
         .                               }
 1,600,808 ( 0.03%)                  }
11,205,656 ( 0.22%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<core::cell::RefMut<lua_rs::table::Table>> (400,202x)
         .                           rt = newrt;
   400,202 ( 0.01%)              }
 1,200,606 ( 0.02%)          }
19,609,898 ( 0.39%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<alloc::rc::Rc<core::cell::RefCell<lua_rs::table::Table>>> (400,202x)
   800,404 ( 0.02%)      }
         .           
         .               /// set a field `k` on table at position `idx` with the last stack value as value
       837 ( 0.00%)      pub(crate) fn set_field(&mut self, idx: isize, k: &str) {
     1,116 ( 0.00%)          debug_assert!(self.stack.len() >= 1);
       279 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (93x)
       837 ( 0.00%)          let value = self.stack.pop().unwrap();
     3,348 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::pop (93x)
     1,860 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/option.rs:core::option::Option<T>::unwrap (93x)
       837 ( 0.00%)          let idx = if idx < 0 && idx > LUA_REGISTRYINDEX {
       540 ( 0.00%)              idx + 1
         .                   } else {
         3 ( 0.00%)              idx
         .                   };
       465 ( 0.00%)          let tvalue = self.index2adr(idx);
    11,331 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::index2adr (93x)
     1,116 ( 0.00%)          debug_assert!(*tvalue != TValue::Nil);
     6,789 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cmp.rs:core::cmp::PartialEq::ne (93x)
       837 ( 0.00%)          let key = TValue::String(Rc::new(k.to_owned()));
    50,167 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/str.rs:alloc::str::<impl alloc::borrow::ToOwned for str>::to_owned (93x)
    34,989 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:alloc::rc::Rc<T>::new (93x)
       651 ( 0.00%)          self.set_tablev(tvalue, key, value);
   327,314 ( 0.01%)  => src/state.rs:lua_rs::state::LuaState::set_tablev (93x)
       279 ( 0.00%)      }
         .           
       835 ( 0.00%)      pub(crate) fn index2adr(&self, index: isize) -> &TValue {
       674 ( 0.00%)          if index > 0 {
         .                       // positive index in the stack
        36 ( 0.00%)              let index = index as usize + self.base;
        96 ( 0.00%)              debug_assert!(index <= self.base_ci[self.ci].top);
       306 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (6x)
        90 ( 0.00%)              if index - 1 >= self.stack.len() {
        18 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (6x)
         .                           return &TValue::Nil;
         .                       }
        78 ( 0.00%)              &self.stack[index - 1]
       306 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (6x)
       498 ( 0.00%)          } else if index > LUA_REGISTRYINDEX {
         .                       // negative index in the stack (count from top)
     1,314 ( 0.00%)              let index = (-index) as usize;
     2,628 ( 0.00%)              debug_assert!(index != 0 && index <= self.stack.len());
       438 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (146x)
     2,482 ( 0.00%)              &self.stack[self.stack.len() - index]
     7,446 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (146x)
       438 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (146x)
         .                   } else {
        90 ( 0.00%)              match index {
        10 ( 0.00%)                  LUA_REGISTRYINDEX => &self.g.registry,
         .                           LUA_ENVIRONINDEX => {
         .                               let stkid = self.base_ci[self.ci].func;
         .                               if let TValue::Function(cl) = &self.stack[stkid] {
         .                                   cl.get_envvalue()
         .                               } else {
         .                                   unreachable!()
         .                               }
         .                           }
        30 ( 0.00%)                  LUA_GLOBALSINDEX => &self.gtvalue,
         .                           _ => {
         .                               // global index - n => return nth upvalue of current Rust closure
         .                               let index = (LUA_GLOBALSINDEX - index) as usize;
         .                               let stkid = self.base_ci[self.ci].func;
         .                               if let TValue::Function(cl) = &self.stack[stkid] {
         .                                   if index <= cl.get_nupvalues() {
         .                                       if let Closure::Rust(cl) = cl.as_ref() {
         .                                           return cl.borrow_upvalue(index - 1);
-- line 382 ----------------------------------------
-- line 384 ----------------------------------------
         .                                   }
         .                                   &TValue::Nil
         .                               } else {
         .                                   unreachable!()
         .                               }
         .                           }
         .                       }
         .                   }
       668 ( 0.00%)      }
         .           
         .               /// put field value `key` from table at `index` on stack
        40 ( 0.00%)      pub(crate) fn get_field(&mut self, index: isize, key: &str) {
        35 ( 0.00%)          let t = self.index2adr(index).clone();
       625 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::index2adr (5x)
       360 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (5x)
        85 ( 0.00%)          Self::get_tablev2(&mut self.stack, &t, &TValue::new_string(key), None);
     6,915 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::get_tablev2 (5x)
     3,534 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::new_string (5x)
     2,490 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (5x)
        20 ( 0.00%)      }
       335 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (5x)
         .           
        30 ( 0.00%)      pub(crate) fn is_table(&self, arg: isize) -> bool {
        30 ( 0.00%)          self.index2adr(arg).is_table()
     1,250 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::index2adr (10x)
        95 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::is_table (10x)
        40 ( 0.00%)      }
        30 ( 0.00%)      pub(crate) fn is_nil(&self, arg: isize) -> bool {
        30 ( 0.00%)          self.index2adr(arg).is_nil()
     1,250 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::index2adr (10x)
        95 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::TValue::is_nil (10x)
        40 ( 0.00%)      }
         .           
        80 ( 0.00%)      pub(crate) fn pop_stack(&mut self, count: usize) {
       176 ( 0.00%)          let newlen = self.stack.len() - count;
        48 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (16x)
        32 ( 0.00%)          self.stack.truncate(newlen);
     1,166 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::truncate (16x)
        32 ( 0.00%)      }
         .           
         .               #[inline]
         .               /// convert an index into an absolute index (-1 => stack.len()-1)
       100 ( 0.00%)      fn index2abs(&self, index: isize) -> usize {
        60 ( 0.00%)          if index < 0 {
       420 ( 0.00%)              self.stack.len() - (-index) as usize
        60 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (20x)
         .                   } else {
         .                       index as usize
         .                   }
        60 ( 0.00%)      }
         .           
        60 ( 0.00%)      pub(crate) fn remove(&mut self, index: isize) {
        60 ( 0.00%)          let index = self.index2abs(index);
       585 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::index2abs (15x)
        90 ( 0.00%)          self.stack.remove(index);
     1,005 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (15x)
       990 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::remove (15x)
        30 ( 0.00%)      }
         .           
         .               /// move the stack top element to position `index`
        20 ( 0.00%)      pub(crate) fn insert(&mut self, index: isize) {
        20 ( 0.00%)          let index = self.index2abs(index);
       195 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::index2abs (5x)
        45 ( 0.00%)          let value = self.stack.pop().unwrap();
       180 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::pop (5x)
       100 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/option.rs:core::option::Option<T>::unwrap (5x)
        35 ( 0.00%)          self.stack.insert(index, value);
       310 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::insert (5x)
        10 ( 0.00%)      }
         .           
         .               /// get a field value from table at `index`. field name is last value on stack
         .               /// result : field value is last value on stack
        70 ( 0.00%)      pub(crate) fn rawget(&mut self, index: isize) {
         .                   let value = {
        80 ( 0.00%)              let key = self.stack.pop().unwrap();
       360 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::pop (10x)
       200 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/option.rs:core::option::Option<T>::unwrap (10x)
        90 ( 0.00%)              let index = if index < 0 && index > LUA_REGISTRYINDEX {
        60 ( 0.00%)                  index + 1
         .                       } else {
         .                           index
         .                       };
        50 ( 0.00%)              let t = self.index2adr(index);
     1,250 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::index2adr (10x)
        70 ( 0.00%)              if let TValue::Table(rct) = t {
        80 ( 0.00%)                  let mut t = rct.borrow_mut();
       840 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:core::cell::RefCell<T>::borrow_mut (10x)
       170 ( 0.00%)                  t.get(&key).unwrap_or(&TValue::Nil).clone()
    14,326 ( 0.00%)  => /home/jice/lua-rs/src/table.rs:lua_rs::table::Table::get (10x)
       450 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (10x)
       190 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/option.rs:core::option::Option<T>::unwrap_or (10x)
        50 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:<core::cell::RefMut<T> as core::ops::deref::DerefMut>::deref_mut (10x)
        40 ( 0.00%)              } else {
       280 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<core::cell::RefMut<lua_rs::table::Table>> (10x)
         .                           unreachable!()
         .                       }
        30 ( 0.00%)          };
     4,980 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (10x)
        70 ( 0.00%)          self.stack.push(value);
       450 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::push (10x)
        20 ( 0.00%)      }
         .           
         .               /// set a field on table at `index`. key and value are the last two objects on stack
        25 ( 0.00%)      pub(crate) fn set_table(&mut self, index: isize) {
        65 ( 0.00%)          debug_assert!(self.stack.len() >= 2);
        15 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (5x)
        80 ( 0.00%)          let value = self.stack.pop().unwrap();
       180 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::pop (5x)
       100 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/option.rs:core::option::Option<T>::unwrap (5x)
        45 ( 0.00%)          let key = self.stack.pop().unwrap();
       180 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::pop (5x)
       100 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/option.rs:core::option::Option<T>::unwrap (5x)
        45 ( 0.00%)          let index = if index < 0 && index > LUA_REGISTRYINDEX {
        30 ( 0.00%)              index + 2
         .                   } else {
         .                       index
         .                   };
        25 ( 0.00%)          let t = self.index2adr(index);
       625 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::index2adr (5x)
        50 ( 0.00%)          self.set_tablev(t, key, value);
    50,815 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::set_tablev (5x)
        20 ( 0.00%)      }
         .           
 2,400,414 ( 0.05%)      pub(crate) fn poscall(&mut self, first_result: u32) -> bool {
         .                   // TODO hooks
 3,600,621 ( 0.07%)          let ci = &self.base_ci[self.ci];
20,403,519 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (400,069x)
         .                   // res == final position of 1st result
   800,138 ( 0.02%)          let mut res = ci.func;
 1,200,207 ( 0.02%)          let wanted = ci.nresults;
         .           
 1,600,276 ( 0.03%)          self.base_ci.pop();
26,004,485 ( 0.52%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::pop (400,069x)
 3,600,621 ( 0.07%)          self.ci -= 1;
 4,000,690 ( 0.08%)          let ci = &self.base_ci[self.ci];
20,403,519 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (400,069x)
   800,138 ( 0.02%)          self.base = ci.base;
   800,138 ( 0.02%)          self.saved_pc = ci.saved_pc;
   400,069 ( 0.01%)          let mut i = wanted;
         .                   // move results to correct place
   800,138 ( 0.02%)          let mut first_result = first_result as usize;
 8,800,552 ( 0.18%)          while i != 0 && first_result < self.stack.len() {
 1,200,018 ( 0.02%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (400,006x)
10,400,078 ( 0.21%)              self.stack[res] = self.stack[first_result].clone();
20,400,153 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (400,003x)
26,800,201 ( 0.54%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (400,003x)
20,400,153 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut (400,003x)
 9,600,168 ( 0.19%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (400,003x)
 3,200,024 ( 0.06%)              res += 1;
 3,200,024 ( 0.06%)              first_result += 1;
 3,200,024 ( 0.06%)              i -= 1;
         .                   }
 1,200,207 ( 0.02%)          while i >0 {
         .                       i=-1;
         .                       self.stack[res] = TValue::Nil;
         .                       res+=1;
         .                   }
 2,400,414 ( 0.05%)          self.stack.resize(res, TValue::Nil);
79,614,833 ( 1.59%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::resize (400,069x)
   800,138 ( 0.02%)          wanted != LUA_MULTRET
 1,600,276 ( 0.03%)      }
         .           
        52 ( 0.00%)      pub(crate) fn find_upval(upvals:&mut Vec<UpVal>, stack: &mut[TValue], level: u32) -> UpVal {
         4 ( 0.00%)          let mut index = 0;
       142 ( 0.00%)          for (i, val) in upvals.iter().enumerate().rev() {
       420 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/adapters/rev.rs:<core::iter::adapters::rev::Rev<I> as core::iter::traits::iterator::Iterator>::next (4x)
       152 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/slice/mod.rs:core::slice::<impl [T]>::iter (4x)
        72 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::deref::Deref>::deref (4x)
        64 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/traits/iterator.rs:core::iter::traits::iterator::Iterator::enumerate (4x)
        60 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/traits/iterator.rs:core::iter::traits::iterator::Iterator::rev (4x)
        32 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/traits/collect.rs:<I as core::iter::traits::collect::IntoIterator>::into_iter (4x)
        12 ( 0.00%)              if val.v < level as StkId {
         6 ( 0.00%)                  index = i + 1;
         .                           break;
         .                       }
         8 ( 0.00%)              if val.v == level as StkId {
         .                           // found a corresponding value
         2 ( 0.00%)                  return val.clone();
       180 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::UpVal as core::clone::Clone>::clone (2x)
         .                       }
         .                   }
         6 ( 0.00%)          let uv = UpVal {
         4 ( 0.00%)              v: level as StkId,
        22 ( 0.00%)              value: stack[level as usize].clone(),
       144 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::TValue as core::clone::Clone>::clone (2x)
         .                   };
        20 ( 0.00%)          upvals.insert(index, uv.clone());
     1,152 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::insert (2x)
       180 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:<lua_rs::object::UpVal as core::clone::Clone>::clone (2x)
        12 ( 0.00%)          uv
        10 ( 0.00%)      }
         .           
   109,161 ( 0.00%)      pub(crate) fn to_number(stack : &mut [TValue], obj: StkId, dst: StkId) -> bool {
   157,677 ( 0.00%)          match &stack[obj] {
    24,258 ( 0.00%)              TValue::Number(_) => true,
         .                       TValue::String(s) => match s.parse::<LuaNumber>() {
         .                           Ok(n) => {
         .                               stack[dst] = TValue::Number(n);
         .                               return true;
         .                           }
         .                           _ => false,
         .                       },
         .                       _ => false,
         .                   }
    60,645 ( 0.00%)      }
         .           
 2,000,305 ( 0.04%)      pub(crate) fn close_func(&mut self, level: StkId) {
 6,801,066 ( 0.14%)          while let Some(uv) = self.open_upval.last() {
 7,201,134 ( 0.14%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::deref::Deref>::deref (400,063x)
 6,000,940 ( 0.12%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/slice/mod.rs:core::slice::<impl [T]>::last (400,063x)
   800,124 ( 0.02%)              if uv.v < level  {
         .                           break;
         .                       }
        16 ( 0.00%)              if uv.v < self.stack.len() {
         6 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (2x)
         .                           self.stack[uv.v] = uv.value.clone();
         .                       }
        12 ( 0.00%)              self.open_upval.pop();
     1,262 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<core::option::Option<lua_rs::object::UpVal>> (2x)
        84 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::pop (2x)
         .                   }
   800,122 ( 0.02%)      }
         .           
         .           }
         .           
         3 ( 0.00%)  fn f_luaopen(state: &mut LuaState, _: ()) -> Result<i32, LuaError> {
         6 ( 0.00%)      let gt = Table::new();
       279 ( 0.00%)  => /home/jice/lua-rs/src/table.rs:lua_rs::table::Table::new (1x)
         2 ( 0.00%)      state.init_stack();
       977 ( 0.00%)  => src/state.rs:lua_rs::state::LuaState::init_stack (1x)
         .               // table of globals
        30 ( 0.00%)      state.l_gt = Rc::new(RefCell::new(gt));
       401 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:alloc::rc::Rc<T>::new (1x)
       394 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<alloc::rc::Rc<core::cell::RefCell<lua_rs::table::Table>>> (1x)
        61 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs:core::cell::RefCell<T>::new (1x)
        14 ( 0.00%)      state.gtvalue = TValue::Table(state.l_gt.clone());
        48 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::clone::Clone>::clone (1x)
        14 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::object::TValue> (1x)
         2 ( 0.00%)      Ok(0)
         4 ( 0.00%)  }
         .           
         4 ( 0.00%)  pub(crate) fn newstate() -> LuaState {
         2 ( 0.00%)      let mut state = LuaState::default();
     6,206 ( 0.00%)  => src/state.rs:<lua_rs::state::LuaState as core::default::Default>::default (1x)
         1 ( 0.00%)      state.allowhook = true;
        12 ( 0.00%)      if rawrunprotected(&mut state, f_luaopen, ()).is_err() {
     2,251 ( 0.00%)  => /home/jice/lua-rs/src/ldo.rs:lua_rs::ldo::rawrunprotected (1x)
        13 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/result.rs:core::result::Result<T,E>::is_err (1x)
         .               } else {
         .               }
         .               state
         2 ( 0.00%)  }

10,461,661 ( 0.21%)  <counts for unidentified lines in src/state.rs>

--------------------------------------------------------------------------------
-- Auto-annotated source: src/ldo.rs
--------------------------------------------------------------------------------
Ir                  

-- line 27 ----------------------------------------
         .           }
         .           
         .           pub struct SParser<T> {
         .               pub z: Option<luaZ::Zio<T>>,
         .               pub name: String,
         .           }
         .           
         .           impl<T> SParser<T> {
        11 ( 0.00%)      pub fn new(z: luaZ::Zio<T>, name: &str) -> Self {
        11 ( 0.00%)          Self {
        13 ( 0.00%)  => ???:0x000000000011a060 (1x)
         9 ( 0.00%)              z: Some(z),
         2 ( 0.00%)              name: name.to_owned(),
       632 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/str.rs:alloc::str::<impl alloc::borrow::ToOwned for str>::to_owned (1x)
         .                   }
         2 ( 0.00%)      }
         .           }
         .           
         .           fn seterrorobj(state: &mut LuaState, errcode: &LuaError) {
         .               match errcode {
         .                   LuaError::ErrorHandlerError => {
         .                       state.push_string("error in error handling");
         .                   }
         .                   LuaError::SyntaxError | LuaError::RuntimeError => {
         .                       let msg = state.stack.last().unwrap().clone();
         .                       state.stack.push(msg);
         .                   }
         .               }
         .           }
         .           
        17 ( 0.00%)  pub fn rawrunprotected<T>(
         .               state: &mut LuaState,
         .               func: Pfunc<T>,
         .               user_data: T,
         .           ) -> Result<i32, LuaError> {
        12 ( 0.00%)      func(state, user_data)
4,994,394,630 (99.93%)  => /home/jice/lua-rs/src/api.rs:lua_rs::api::f_call (1x)
 2,162,614 ( 0.04%)  => src/ldo.rs:lua_rs::ldo::f_parser (1x)
     2,241 ( 0.00%)  => /home/jice/lua-rs/src/state.rs:lua_rs::state::f_luaopen (1x)
         9 ( 0.00%)  }
         .           
         .           impl LuaState {
         .               ///  Call a function (Rust or Lua). The function to be called is at stack[cl_stkid].
         .               ///  The arguments are on the stack, right after the function.
         .               ///  When returns, all the results are on the stack, starting at the original
         .               ///  function position.
        42 ( 0.00%)      pub(crate) fn dcall(&mut self, cl_stkid: StkId, nresults: i32) -> Result<(), LuaError> {
        42 ( 0.00%)          self.n_rcalls += 1;
        18 ( 0.00%)          if self.n_rcalls >= LUAI_MAXRCALLS {
         .                       if self.n_rcalls == LUAI_MAXRCALLS {
         .                           return self.run_error("Rust stack overflow");
         .                       } else if self.n_rcalls >= LUAI_MAXRCALLS + (LUAI_MAXRCALLS >> 3) {
         .                           // error while handing stack error
         .                           return Err(LuaError::ErrorHandlerError);
         .                       }
         .                   }
        54 ( 0.00%)          if let TValue::Function(_cl) = &self.stack[cl_stkid] {
       255 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (5x)
       113 ( 0.00%)              if let PrecallStatus::Lua = self.dprecall(cl_stkid, nresults)? {
   737,857 ( 0.01%)  => src/ldo.rs:lua_rs::ldo::<impl lua_rs::state::LuaState>::dprecall (5x)
        90 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/result.rs:<core::result::Result<T,E> as core::ops::try_trait::Try>::branch (5x)
        11 ( 0.00%)                  self.vexecute(1)?;
4,994,390,314 (99.93%)  => /home/jice/lua-rs/src/vm.rs:lua_rs::vm::<impl lua_rs::state::LuaState>::vexecute (1x)
        12 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/result.rs:<core::result::Result<T,E> as core::ops::try_trait::Try>::branch (1x)
         .                       }
         .                   }
        60 ( 0.00%)          self.n_rcalls -= 1;
         6 ( 0.00%)          Ok(())
        24 ( 0.00%)      }
         .           
 2,800,483 ( 0.06%)      pub(crate) fn dprecall(
         .                   &mut self,
         .                   cl_stkid: StkId,
         .                   nresults: i32,
         .               ) -> Result<PrecallStatus, LuaError> {
 2,400,414 ( 0.05%)          let cl_stkid = match &self.stack[cl_stkid] {
20,403,468 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (400,068x)
         .                       TValue::Function(_) => cl_stkid,
         .                       _ => {
         .                           // func' is not a function. check the `function' metamethod
         .                           // TODO
         .                           //self.try_func_tag_method(cl_stkid)?
         .                           luaG::type_error(self, cl_stkid, "call")?;
         .                           unreachable!()
         .                       }
         .                   };
 3,600,621 ( 0.07%)          let cl = if let TValue::Function(cl) = &self.stack[cl_stkid] {
20,403,468 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (400,068x)
 1,600,276 ( 0.03%)              cl.clone()
19,203,264 ( 0.38%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::clone::Clone>::clone (400,068x)
         .                   } else {
         .                       unreachable!()
         .                   };
 4,000,690 ( 0.08%)          self.base_ci[self.ci].saved_pc = self.saved_pc;
20,403,468 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::IndexMut<I>>::index_mut (400,068x)
 2,800,483 ( 0.06%)          match cl.as_ref() {
 2,400,408 ( 0.05%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::convert::AsRef<T>>::as_ref (400,068x)
 1,600,244 ( 0.03%)              Closure::Lua(cl) => {
         .                           // Lua function. prepare its call
 4,000,609 ( 0.08%)                  let base = if self.protos[cl.proto].is_vararg {
20,403,111 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (400,061x)
         .                               // vararg function
        21 ( 0.00%)                      let nargs = self.stack.len() - cl_stkid - 1;
         3 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (1x)
         5 ( 0.00%)                      self.adjust_varargs(cl.proto, nargs)
       183 ( 0.00%)  => src/ldo.rs:lua_rs::ldo::<impl lua_rs::state::LuaState>::adjust_varargs (1x)
         .                           } else {
         .                               // no varargs
 2,800,420 ( 0.06%)                      let base = cl_stkid + 1;
 8,001,200 ( 0.16%)                      if self.stack.len() > base + self.protos[cl.proto].numparams {
20,403,060 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (400,060x)
 1,200,180 ( 0.02%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (400,060x)
         .                                   panic!("cannot truncate stack in dprecall");
         .                                   //self.stack.truncate(base + cl.proto.numparams);
         .                               }
   400,060 ( 0.01%)                      base
         .                           };
   800,122 ( 0.02%)                  let mut ci = CallInfo::default();
16,802,562 ( 0.34%)  => /home/jice/lua-rs/src/state.rs:<lua_rs::state::CallInfo as core::default::Default>::default (400,061x)
   400,061 ( 0.01%)                  ci.func = cl_stkid;
   800,122 ( 0.02%)                  ci.base = base;
   800,122 ( 0.02%)                  self.base = base;
 6,000,915 ( 0.12%)                  ci.top = base + self.protos[cl.proto].maxstacksize;
20,403,111 ( 0.41%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (400,061x)
   400,061 ( 0.01%)                  self.saved_pc = 0;
   400,061 ( 0.01%)                  ci.nresults = nresults;
 2,800,427 ( 0.06%)                  self.stack.resize(ci.top, TValue::Nil);
122,477,701 ( 2.45%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::resize (400,061x)
 4,400,671 ( 0.09%)                  self.base_ci.push(ci);
28,805,648 ( 0.58%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::push (400,061x)
 2,800,427 ( 0.06%)                  self.ci += 1;
         .                           // TODO handle hooks
 1,600,244 ( 0.03%)                  return Ok(PrecallStatus::Lua);
         .                       }
        32 ( 0.00%)              Closure::Rust(cl) => {
         .                           // this is a Rust function, call it
        16 ( 0.00%)                  let mut ci = CallInfo::default();
       294 ( 0.00%)  => /home/jice/lua-rs/src/state.rs:<lua_rs::state::CallInfo as core::default::Default>::default (7x)
         8 ( 0.00%)                  ci.func = cl_stkid;
        48 ( 0.00%)                  self.base = cl_stkid + 1;
        56 ( 0.00%)                  ci.base = cl_stkid + 1;
         8 ( 0.00%)                  ci.nresults = nresults;
        88 ( 0.00%)                  ci.top = self.stack.len() + LUA_MINSTACK;
        21 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (7x)
        88 ( 0.00%)                  self.base_ci.push(ci);
       504 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::push (7x)
        56 ( 0.00%)                  self.ci += 1;
         .                           // TODO handle hooks
       224 ( 0.00%)                  let n = (cl.f)(self).map_err(|_| LuaError::RuntimeError)?;
   244,542 ( 0.00%)  => /home/jice/lua-rs/src/libs/base.rs:lua_rs::libs::base::lib_open_base (1x)
   229,934 ( 0.00%)  => /home/jice/lua-rs/src/libs/maths.rs:lua_rs::libs::maths::lib_open_math (1x)
   164,648 ( 0.00%)  => /home/jice/lua-rs/src/libs/string.rs:lua_rs::libs::string::lib_open_string (1x)
    86,633 ( 0.00%)  => /home/jice/lua-rs/src/libs/io.rs:lua_rs::libs::io::lib_open_io (1x)
     8,134 ( 0.00%)  => /home/jice/lua-rs/src/libs/base.rs:lua_rs::libs::base::luab_print (1x)
     7,824 ( 0.00%)  => /home/jice/lua-rs/src/libs/string.rs:lua_rs::libs::string::str_format (1x)
       359 ( 0.00%)  => /home/jice/lua-rs/src/libs/maths.rs:lua_rs::libs::maths::math_sqrt (1x)
       126 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/result.rs:core::result::Result<T,E>::map_err (7x)
       105 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/result.rs:<core::result::Result<T,E> as core::ops::try_trait::Try>::branch (7x)
        24 ( 0.00%)                  if n < 0 {
         .                               return Ok(PrecallStatus::RustYield);
         .                           } else {
       112 ( 0.00%)                      self.poscall(self.stack.len() as u32 - n as u32);
        21 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (7x)
     6,453 ( 0.00%)  => /home/jice/lua-rs/src/state.rs:lua_rs::state::LuaState::poscall (7x)
        40 ( 0.00%)                      return Ok(PrecallStatus::Rust);
         .                           }
         .                       }
         .                   }
 2,800,483 ( 0.06%)      }
19,605,024 ( 0.39%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<alloc::rc::Rc<lua_rs::object::Closure>> (400,068x)
         .           
         6 ( 0.00%)      pub(crate) fn adjust_varargs(&mut self, proto: ProtoRef, nargs: usize) -> usize {
         8 ( 0.00%)          let nfix_args = self.protos[proto].numparams;
        51 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index (1x)
        14 ( 0.00%)          for _ in nargs..nfix_args {
        28 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/range.rs:core::iter::range::<impl core::iter::traits::iterator::Iterator for core::ops::range::Range<A>>::next (1x)
         5 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/traits/collect.rs:<I as core::iter::traits::collect::IntoIterator>::into_iter (1x)
         .                       self.stack.push(TValue::Nil);
         .                   }
         .                   // move fixed parameters to final position
         5 ( 0.00%)          let base = self.stack.len(); // final position of first argument
         3 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (1x)
         9 ( 0.00%)          let fixed_pos = base - nargs; // first fixed argument
        15 ( 0.00%)          for i in 0..nfix_args {
        28 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/range.rs:core::iter::range::<impl core::iter::traits::iterator::Iterator for core::ops::range::Range<A>>::next (1x)
         5 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/traits/collect.rs:<I as core::iter::traits::collect::IntoIterator>::into_iter (1x)
         .                       let value = self.stack.remove(fixed_pos + i);
         .                       self.stack.insert(fixed_pos + i, TValue::Nil);
         .                       self.stack.push(value);
         .                   }
         .                   base
         2 ( 0.00%)      }
         .               pub(crate) fn try_func_tag_method(&self, _cl_stkid: StkId) -> Result<StkId, LuaError> {
         .                   todo!()
         .               }
         .           }
         .           
        16 ( 0.00%)  pub fn pcall<T>(
         .               state: &mut LuaState,
         .               func: Pfunc<T>,
         .               u: T,
         .               old_top: StkId,
         .               ef: StkId,
         .           ) -> Result<i32, LuaError> {
         .               let old_errfunc;
         .               let old_allowhook;
         .               let old_ci;
         .               let old_n_ccalls;
         .               {
         6 ( 0.00%)          old_n_ccalls = state.n_rcalls;
         6 ( 0.00%)          old_ci = state.ci;
         8 ( 0.00%)          old_allowhook = state.allowhook;
         6 ( 0.00%)          old_errfunc = state.errfunc;
         2 ( 0.00%)          state.errfunc = ef;
         .               }
         8 ( 0.00%)      let status = rawrunprotected(state, func, u);
4,996,557,272 (99.98%)  => src/ldo.rs:lua_rs::ldo::rawrunprotected (2x)
        10 ( 0.00%)      if let Err(e) = &status {
         .                   state.close_func(old_top);
         .                   seterrorobj(state, e);
         .                   state.n_rcalls = old_n_ccalls;
         .                   state.ci = old_ci;
         .                   state.base = state.base_ci[state.ci].base;
         .                   state.saved_pc = state.base_ci[state.ci].saved_pc;
         .                   state.allowhook = old_allowhook;
         .               }
         2 ( 0.00%)      state.errfunc = old_errfunc;
         .               status
         6 ( 0.00%)  }
         .           
         5 ( 0.00%)  fn f_parser<T>(state: &mut LuaState, parser: &mut SParser<T>) -> Result<i32, LuaError> {
        10 ( 0.00%)      let c = if let Some(ref mut z) = parser.z {
         3 ( 0.00%)          z.look_ahead(state)
   150,045 ( 0.00%)  => /home/jice/lua-rs/src/zio.rs:lua_rs::zio::Zio<T>::look_ahead (1x)
         .               } else {
         .                   unreachable!()
         .               };
        34 ( 0.00%)      let proto = if c == LUA_SIGNATURE.chars().next() {
 2,004,379 ( 0.04%)  => /home/jice/lua-rs/src/parser.rs:lua_rs::parser::parser (1x)
       120 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/str/iter.rs:<core::str::iter::Chars as core::iter::traits::iterator::Iterator>::next (1x)
        60 ( 0.00%)  => ./string/../sysdeps/x86_64/multiarch/memmove-vec-unaligned-erms.S:__memcpy_avx_unaligned_erms (2x)
        84 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/result.rs:<core::result::Result<T,E> as core::ops::try_trait::Try>::branch (1x)
        51 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/option.rs:<core::option::Option<T> as core::cmp::PartialEq>::eq (1x)
        43 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/str/mod.rs:core::str::<impl str>::chars (1x)
         .                   luaU::undump
         2 ( 0.00%)      } else {
         .                   luaY::parser
         .               }(state, parser)?;
         3 ( 0.00%)      let nups = proto.nups;
         6 ( 0.00%)      let protoid = state.protos.len();
         3 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (1x)
        14 ( 0.00%)      state.protos.push(proto);
     7,035 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::push (1x)
        30 ( 0.00%)  => ./string/../sysdeps/x86_64/multiarch/memmove-vec-unaligned-erms.S:__memcpy_avx_unaligned_erms (1x)
        10 ( 0.00%)      let mut luacl = LClosure::new(protoid, Rc::clone(&state.l_gt));
        95 ( 0.00%)  => /home/jice/lua-rs/src/object.rs:lua_rs::object::LClosure::new (1x)
        48 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:<alloc::rc::Rc<T> as core::clone::Clone>::clone (1x)
        21 ( 0.00%)      for _ in 0..nups {
        28 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/range.rs:core::iter::range::<impl core::iter::traits::iterator::Iterator for core::ops::range::Range<A>>::next (1x)
         5 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/traits/collect.rs:<I as core::iter::traits::collect::IntoIterator>::into_iter (1x)
         .                   luacl.upvalues.push(UpVal::default());
         .               }
        18 ( 0.00%)      let cl = Closure::Lua(luacl);
        13 ( 0.00%)      state.stack.push(TValue::Function(Rc::new(cl)));
        45 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::push (1x)
       372 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs:alloc::rc::Rc<T>::new (1x)
         2 ( 0.00%)      Ok(0)
         6 ( 0.00%)  }
         .           
         5 ( 0.00%)  pub fn protected_parser<T>(
         .               state: &mut LuaState,
         .               zio: luaZ::Zio<T>,
         .               chunk_name: &str,
         .           ) -> Result<i32, LuaError> {
         3 ( 0.00%)      let mut p = SParser::new(zio, chunk_name);
       681 ( 0.00%)  => src/ldo.rs:lua_rs::ldo::SParser<T>::new (1x)
         6 ( 0.00%)      let top = state.stack.len();
         3 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs:alloc::vec::Vec<T,A>::len (1x)
         2 ( 0.00%)      let errfunc = state.errfunc;
         9 ( 0.00%)      pcall(state, f_parser, &mut p, top, errfunc)
 2,162,665 ( 0.04%)  => src/ldo.rs:lua_rs::ldo::pcall (1x)
         5 ( 0.00%)  }
       256 ( 0.00%)  => /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs:core::ptr::drop_in_place<lua_rs::ldo::SParser<&str>> (1x)

12,802,270 ( 0.26%)  <counts for unidentified lines in src/ldo.rs>

--------------------------------------------------------------------------------
The following files chosen for auto-annotation could not be found:
--------------------------------------------------------------------------------
  ./string/../sysdeps/x86_64/multiarch/memmove-vec-unaligned-erms.S
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/raw_vec.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/rc.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/mod.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/alloc/src/vec/set_len_on_drop.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cell.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/cmp.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/default.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/iter/range.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/mem/mod.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/num/uint_macros.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/const_ptr.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/metadata.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mod.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/mut_ptr.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/non_null.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/ptr/unique.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/result.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/slice/index.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/slice/mod.rs
  /rustc/90c541806f23a127002de5b4038be731ba1458ca/library/core/src/slice/raw.rs

--------------------------------------------------------------------------------
Ir                     
--------------------------------------------------------------------------------
1,898,227,283 (37.98%)  events annotated

```

