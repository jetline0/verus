#![feature(concat_idents)]
#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

// Run each test for both AtomicInvariant/open_atomic_invariant and LocalInvariant/open_local_invariant

macro_rules! test_both {
    ($name:ident $name2:ident $test:expr => $p:pat) => {
        test_verify_one_file! {
            #[test] $name $test => $p
        }

        test_verify_one_file! {
            #[test] $name2 ($test
                .replace("AtomicInvariant", "LocalInvariant")
                .replace("open_atomic_invariant", "open_local_invariant")) => $p
        }
    };
    ($name:ident $name2:ident $test:expr => $p:pat => $e:expr) => {
        test_verify_one_file! {
            #[test] $name $test => $p => $e
        }

        test_verify_one_file! {
            #[test] $name2 ($test
                .replace("AtomicInvariant", "LocalInvariant")
                .replace("open_atomic_invariant", "open_local_invariant")) => $p => $e
        }
    };
}

test_both! {
    basic_usage basic_usage_local verus_code! {
        use vstd::invariant::*;

        struct Foo { }

        pub fn X<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<AtomicInvariant<A, u8, B>>)
            requires
                i.inv(0),
        {
            open_atomic_invariant!(&i => inner => {
                let tracked x = Foo { };
                let tracked x = Foo { };
                proof { inner = 0u8; }
            });
        }
    } => Ok(())
}

test_both! {
    basic_usage2 basic_usage2_local verus_code! {
        use vstd::invariant::*;

        pub fn X<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<AtomicInvariant<A, u8, B>>) {
            open_atomic_invariant!(&i => inner => {
            });
        }
    } => Ok(())
}

test_both! {
    inv_fail inv_fail_local verus_code! {
        use vstd::invariant::*;
        struct Foo { }
        pub fn X<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<AtomicInvariant<A, u8, B>>) {
            open_atomic_invariant!(&i => inner => {
                let tracked x = Foo { };
                let tracked x = Foo { };
                proof { inner = 0u8; }
            }); // FAILS
        }
    } => Err(err) => assert_one_fails(err)
}

test_both! {
    nested_failure nested_failure_local verus_code! {
        use vstd::invariant::*;
        pub fn nested<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<AtomicInvariant<A, u8, B>>)
            requires
                i.inv(0),
        {
            open_atomic_invariant!(&i => inner => { // FAILS
                open_atomic_invariant!(&i => inner2 => {
                    proof { inner2 = 0u8; }
                });
                proof { inner = 0u8; }
            });
        }
    } => Err(err) => assert_one_fails(err)
}

test_both! {
    nested_good nested_good_local verus_code! {
        use vstd::invariant::*;
        pub fn nested_good<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<AtomicInvariant<A, u8, B>>, Tracked(j): Tracked<AtomicInvariant<A, u8, B>>)
            requires
                i.inv(0),
                j.inv(1),
                i.namespace() == 0,
                j.namespace() == 1,
        {
            open_atomic_invariant!(&i => inner => {
                proof { inner = 0u8; }
                open_atomic_invariant!(&j => inner => {
                    proof { inner = 1u8; }
                });
            });
        }
    } => Ok(())
}

test_both! {
    full_call_empty full_call_empty_local verus_code! {
        use vstd::invariant::*;
        pub proof fn callee_mask_empty()
          opens_invariants none // will not open any invariant
        {
        }
        pub fn t1<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<AtomicInvariant<A, u8, B>>) {
          open_atomic_invariant!(&i => inner => {
            proof { callee_mask_empty(); }
          });
        }
    } => Ok(())
}

test_both! {
    open_call_full open_call_full_local verus_code! {
        use vstd::invariant::*;
        pub proof fn callee_mask_full()
          opens_invariants any // can open any invariant
        {
        }
        pub fn t2<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<AtomicInvariant<A, u8, B>>) {
          open_atomic_invariant!(&i => inner => { // FAILS
            proof { callee_mask_full(); }
          });
        }
    } => Err(err) => assert_one_fails(err)
}

test_both! {
    empty_open empty_open_local verus_code! {
        use vstd::invariant::*;
        pub proof fn callee_mask_empty()
          opens_invariants none // will not open any invariant
        {
        }
        pub fn t3<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<AtomicInvariant<A, u8, B>>)
          opens_invariants none
        {
          open_atomic_invariant!(&i => inner => { // FAILS
          });
        }
    } => Err(err) => assert_one_fails(err)
}

// mode stuff

test_both! {
    open_inv_in_spec open_inv_in_spec_local verus_code! {
        use vstd::invariant::*;

        pub closed spec fn open_inv_in_spec<A, B: InvariantPredicate<A, u8>>(i: AtomicInvariant<A, u8, B>) {
          open_atomic_invariant!(&i => inner => {
          });
        }
    } => Err(err) => assert_vir_error_msg(err, "Cannot open invariant in Spec mode")
}

test_both! {
    inv_header_in_spec inv_header_in_spec_local verus_code! {
        use vstd::invariant::*;

        pub closed spec fn inv_header_in_spec<A, B: InvariantPredicate<A, u8>>(i: AtomicInvariant<A, u8, B>)
          opens_invariants any
        {
        }
    } => Err(err) => assert_vir_error_msg(err, "invariants cannot be opened in spec functions")
}

test_both! {
    open_inv_in_proof open_inv_in_proof_local verus_code! {
        use vstd::invariant::*;

        pub proof fn open_inv_in_proof<A, B: InvariantPredicate<A, u8>>(tracked i: AtomicInvariant<A, u8, B>)
          opens_invariants any
        {
          open_atomic_invariant!(&i => inner => {
          });
        }
    } => Ok(())
}

test_both! {
    inv_exec inv_exec_local verus_code! {
        use vstd::invariant::*;

        // this is no longer an error because the `&i` is processed as a 'proof block' argument
        // so the `i` is automatically upgraded to proof mode

        pub fn X<A, B: InvariantPredicate<A, u8>>(i: AtomicInvariant<A, u8, B>) {
            open_atomic_invariant!(&i => inner => {
            });
        }
    } => Ok(())
}

test_both! {
    inv_cannot_be_spec inv_cannot_be_spec_local verus_code! {
        use vstd::invariant::*;

        pub fn X<A, B: InvariantPredicate<A, u8>>(Ghost(i): Ghost<AtomicInvariant<A, u8, B>>) {
            open_atomic_invariant!(&i => inner => {
            });
        }

    } => Err(err) => assert_vir_error_msg(err, "Invariant must be Proof mode")
}

// This test doesn't apply to LocalInvariant
test_verify_one_file! {
    #[test] exec_code_in_inv_block verus_code! {
        use vstd::invariant::*;

        pub fn exec_fn() { }

        pub fn X<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<AtomicInvariant<A, u8, B>>) {
            open_atomic_invariant!(&i => inner => {
                exec_fn();
            });
        }
    } => Err(err) => assert_vir_error_msg(err, "open_atomic_invariant cannot contain non-atomic operations")
}

test_both! {
    inv_lifetime inv_lifetime_local verus_code! {
        use vstd::invariant::*;

        proof fn throw_away<A, B: InvariantPredicate<A, u8>>(tracked i: AtomicInvariant<A, u8, B>) {
        }

        pub fn do_nothing<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<AtomicInvariant<A, u8, B>>)
          requires
            i.inv(0),
        {
          open_atomic_invariant!(&i => inner => {
            proof { throw_away(i); }
          });
        }
    } => Err(err) => assert_vir_error_msg(err, "cannot move out of `i` because it is borrowed")
}

test_both! {
    return_early return_early_local verus_code! {
        use vstd::invariant::*;

        pub fn blah<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<AtomicInvariant<A, u8, B>>) {
          open_atomic_invariant!(&i => inner => {
            return;
          });
        }
    } => Err(err) => assert_vir_error_msg(err, "invariant block might exit early")
}

test_both! {
    return_early_nested return_early_nested_local verus_code! {
        use vstd::invariant::*;

        pub fn blah<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<AtomicInvariant<A, u8, B>>, Tracked(j): Tracked<AtomicInvariant<A, u8, B>>) {
          open_atomic_invariant!(&i => inner => {
            open_atomic_invariant!(&j => inner => {
              return;
            });
          });
        }
    } => Err(err) => assert_vir_error_msg(err, "invariant block might exit early")
}

test_both! {
    break_early break_early_local verus_code! {
        use vstd::invariant::*;

        pub fn blah<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<AtomicInvariant<A, u8, B>>) {
          let mut idx = 0;
          while idx < 5 {
            open_atomic_invariant!(&i => inner => {
              break;
            });
          }
        }
    } => Err(err) => assert_vir_error_msg(err, "invariant block might exit early")
}

test_both! {
    continue_early continue_early_local verus_code! {
        use vstd::invariant::*;

        pub fn blah<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<AtomicInvariant<A, u8, B>>) {
          let mut idx = 0;
          while idx < 5 {
            open_atomic_invariant!(&i => inner => {
              break;
            });
          }
        }
    } => Err(err) => assert_vir_error_msg(err, "invariant block might exit early")
}

test_both! {
    return_early_proof return_early_proof_local verus_code! {
        use vstd::invariant::*;

        pub proof fn blah<A, B: InvariantPredicate<A, u8>>(tracked i: AtomicInvariant<A, u8, B>) {
          open_atomic_invariant!(&i => inner => {
            return;
          });
        }
    } => Err(err) => assert_vir_error_msg(err, "invariant block might exit early")
}

test_both! {
    break_early_proof break_early_proof_local verus_code! {
        use vstd::invariant::*;

        pub proof fn blah<A, B: InvariantPredicate<A, u8>>(tracked i: AtomicInvariant<A, u8, B>) {
          let mut idx: int = 0;
          while idx < 5 {
            open_atomic_invariant!(&i => inner => {
              break;
            });
          }
        }

    } => Err(err) => assert_vir_error_msg(err, "invariant block might exit early")
}

test_both! {
    continue_early_proof continue_early_proof_local verus_code! {
        use vstd::invariant::*;

        pub proof fn blah<A, B: InvariantPredicate<A, u8>>(tracked i: AtomicInvariant<A, u8, B>) {
          let mut idx: int = 0;
          while idx < 5 {
            open_atomic_invariant!(&i => inner => {
              break;
            });
          }
        }

    } => Err(err) => assert_vir_error_msg(err, "invariant block might exit early")
}

// Check that we can't open a AtomicInvariant with open_local_invariant and vice-versa

test_verify_one_file! {
    #[test] mixup1 verus_code! {
        use vstd::invariant::*;

        pub fn X<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<LocalInvariant<A, u8, B>>) {
            open_atomic_invariant!(&i => inner => {
            });
        }
    } => Err(err) => assert_rust_error_msg(err, "mismatched types")
}

test_verify_one_file! {
    #[test] mixup2 verus_code! {
        use vstd::invariant::*;

        pub fn X<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<AtomicInvariant<A, u8, B>>) {
            open_local_invariant!(&i => inner => {
            });
        }
    } => Err(err) => assert_rust_error_msg(err, "mismatched types")
}

test_verify_one_file! {
    #[test] nest_local_loop_local verus_code! {
        use vstd::invariant::*;

        pub fn X<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<LocalInvariant<A, u8, B>>, Tracked(j): Tracked<LocalInvariant<A, u8, B>>) {
            open_local_invariant!(&i => inner => { // FAILS
                let mut idx: u64 = 0;
                while idx < 5 {
                    open_local_invariant!(&j => jnner => {
                    });
                    idx = idx + 1;
                }
            });
        }
    } => Err(err) => assert_one_fails(err)
}

test_verify_one_file! {
    #[test] never_terminate_in_invariant verus_code! {
        use vstd::invariant::*;

        pub fn X<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<LocalInvariant<A, u8, B>>) {
            open_local_invariant!(&i => inner => {
                proof { inner = 7u8; }
                loop { }
            });
        }
    } => Ok(_err) => { /* allow unreachable warnings */ }
}

test_verify_one_file! {
    #[test] opens_invariants_concrete verus_code! {
        use vstd::invariant::*;

        fn stuff()
          opens_invariants [ 0int ]
        {
            stuff2();
        }

        fn stuff2()
          opens_invariants [ 0int, 1int ] // FAILS
        {
        }

        fn stuff3()
          opens_invariants [ 0int ]
        {
        }

        fn stuff4()
          opens_invariants [ 0int, 1int ]
        {
            stuff3();
        }

        fn stuff5()
        {
            stuff3();
        }

        fn stuff6()
          opens_invariants [ 0int, 1int ]
        {
            stuff5(); // FAILS
        }

        fn symbolic(x: u8)
          opens_invariants [ x ] // FAILS
        {
        }

        fn symbolic_caller(x: u8, y: u8)
          opens_invariants [ y ]
        {
          symbolic(x);
        }

        fn symbolic2(x: u8)
          opens_invariants [ x ]
        {
        }

        fn symbolic2_caller(x: u8, y: u8)
          requires x == y,
          opens_invariants [ y ]
        {
          symbolic2(x);
        }

        fn test_inside_open()
          opens_invariants [ 1int ]
        {
        }

        fn test_inside_open_caller<A, B: InvariantPredicate<A, u8>>(Tracked(i): Tracked<LocalInvariant<A, u8, B>>)
          requires i.namespace() == 1,
          opens_invariants [ 1int ]
        {
            open_local_invariant!(&i => inner => { // FAILS
                test_inside_open();
            });
        }

    } => Err(err) => assert_fails(err, 4)
}

test_verify_one_file! {
    #[test] opens_invariants_old_fail verus_code! {
        fn stuff6(x: &mut u8)
          opens_invariants [ ((*x) as int) ]
        {
        }
    } => Err(err) => assert_vir_error_msg(err, "in opens_invariants clause, use `old(x)` to refer to the pre-state of an &mut variable")
}

test_verify_one_file! {
    #[test] opens_invariants_wrong_type verus_code! {
        fn stuff6(x: &mut u8)
          opens_invariants [ true ]
        {
        }
    } => Err(err) => assert_vir_error_msg(err, "opens_invariants needs an int expression")
}

test_verify_one_file! {
    #[test] opens_invariants_private_fn verus_code! {
        spec fn some_inv() -> int { 5 }

        pub fn test(x: &mut u8)
          opens_invariants [ some_inv() ]
        {
        }
    } => Err(err) => assert_vir_error_msg(err, "in 'opens_invariants' clause of public function, cannot refer to private function")
}
