use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

wasmtime_wiggle::from_witx!({
    witx: ["$CARGO_MANIFEST_DIR/tests/atoms.witx"],
    async_: {
        atoms::{double_int_return_float}
    }
});

wasmtime_wiggle::wasmtime_integration!({
    target: crate,
    witx: ["$CARGO_MANIFEST_DIR/tests/atoms.witx"],
    ctx: Ctx,
    modules: { atoms => { name: Atoms } },
    async_: {
        atoms::{double_int_return_float}
    }
});

struct Ctx;
impl wiggle::GuestErrorType for types::Errno {
    fn success() -> Self {
        types::Errno::Ok
    }
}

fn async_store() -> wasmtime::Store {
    let engine = wasmtime::Engine::default();
    wasmtime::Store::new_async(&engine)
}

#[wasmtime_wiggle::async_trait(?Send)]
impl atoms::Atoms for Ctx {
    fn int_float_args(&self, an_int: u32, an_float: f32) -> Result<(), types::Errno> {
        println!("INT FLOAT ARGS: {} {}", an_int, an_float);
        Ok(())
    }
    async fn double_int_return_float(
        &self,
        an_int: u32,
    ) -> Result<types::AliasToFloat, types::Errno> {
        Ok((an_int as f32) * 2.0)
    }
}

// There's nothing meaningful to test here - this just demonstrates the test machinery
#[test]
fn test_sync_host_func() {
    let store = async_store();
    let ctx = Rc::new(RefCell::new(Ctx));
    let atoms = Atoms::new(&store, ctx.clone());

    let results = atoms
        .int_float_args
        .call(&[0i32.into(), 123.45f32.into()])
        .unwrap();

    assert_eq!(results.len(), 1, "one return value");
    assert_eq!(
        results[0].unwrap_i32(),
        types::Errno::Ok as i32,
        "int_float_args errno"
    );
}

/*
#[derive(Debug)]
struct DoubleIntExercise {
    pub input: u32,
    pub return_loc: MemArea,
}

impl DoubleIntExercise {
    pub fn test(&self) {
        let ctx = Ctx;
        let host_memory = HostMemory::new();

        let e = run(atoms::double_int_return_float(
            &ctx,
            &host_memory,
            self.input as i32,
            self.return_loc.ptr as i32,
        ));

        let return_val = host_memory
            .ptr::<types::AliasToFloat>(self.return_loc.ptr)
            .read()
            .expect("failed to read return");
        assert_eq!(e, Ok(types::Errno::Ok as i32), "errno");
        assert_eq!(return_val, (self.input as f32) * 2.0, "return val");
    }

    pub fn strat() -> BoxedStrategy<Self> {
        (prop::num::u32::ANY, HostMemory::mem_area_strat(4))
            .prop_map(|(input, return_loc)| DoubleIntExercise { input, return_loc })
            .boxed()
    }
}

proptest! {
    #[test]
    fn double_int_return_float(e in DoubleIntExercise::strat()) {
        e.test()
    }
}
*/

fn run<F: Future>(future: F) -> F::Output {
    let mut f = Pin::from(Box::new(future));
    let waker = dummy_waker();
    let mut cx = Context::from_waker(&waker);
    loop {
        match f.as_mut().poll(&mut cx) {
            Poll::Ready(val) => break val,
            Poll::Pending => {}
        }
    }
}

fn dummy_waker() -> Waker {
    return unsafe { Waker::from_raw(clone(5 as *const _)) };

    unsafe fn clone(ptr: *const ()) -> RawWaker {
        assert_eq!(ptr as usize, 5);
        const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);
        RawWaker::new(ptr, &VTABLE)
    }

    unsafe fn wake(ptr: *const ()) {
        assert_eq!(ptr as usize, 5);
    }

    unsafe fn wake_by_ref(ptr: *const ()) {
        assert_eq!(ptr as usize, 5);
    }

    unsafe fn drop(ptr: *const ()) {
        assert_eq!(ptr as usize, 5);
    }
}
