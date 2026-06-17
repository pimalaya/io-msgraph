//! Coroutine driver: the `MsgraphCoroutine` trait, its `MsgraphYield` /
//! `MsgraphCoroutineState`, and the `msgraph_try!` macro (the coroutine
//! equivalent of `?`).

use alloc::vec::Vec;

#[derive(Debug)]
pub enum MsgraphCoroutineState<Y, R> {
    Yielded(Y),
    Complete(R),
}

pub trait MsgraphCoroutine {
    type Yield;
    type Return;

    fn resume(&mut self, arg: Option<&[u8]>) -> MsgraphCoroutineState<Self::Yield, Self::Return>;
}

#[derive(Debug)]
pub enum MsgraphYield {
    WantsRead,
    WantsWrite(Vec<u8>),
}

#[macro_export]
macro_rules! msgraph_try {
    ($coroutine:expr, $arg:expr $(,)?) => {
        match $crate::coroutine::MsgraphCoroutine::resume($coroutine, $arg) {
            $crate::coroutine::MsgraphCoroutineState::Yielded(y) => {
                return $crate::coroutine::MsgraphCoroutineState::Yielded(y.into());
            }
            $crate::coroutine::MsgraphCoroutineState::Complete(Err(err)) => {
                log::trace!("error during coroutine execution: {err}");
                return $crate::coroutine::MsgraphCoroutineState::Complete(Err(err.into()));
            }
            $crate::coroutine::MsgraphCoroutineState::Complete(Ok(value)) => value,
        }
    };
}
