use crate::{across_ffi::callbacks, sys, Discord, DiscordResult};
use crossbeam_channel::{Receiver, Sender};
use std::ffi::c_void;
use std::marker::PhantomData;

pub(crate) trait AnyCallback {
    fn is_ready(&self) -> bool;
    fn run(&mut self, _: &mut Discord);
}

pub(crate) struct ResultCallback<F>
where
    F: FnMut(&mut Discord, DiscordResult<()>),
{
    pub(crate) callback: F,
    pub(crate) receiver: Receiver<DiscordResult<()>>,
}

impl<F> ResultCallback<F>
where
    F: FnMut(&mut Discord, DiscordResult<()>),
{
    pub(crate) fn new(callback: F) -> (Self, Sender<DiscordResult<()>>) {
        let (sender, receiver) = crossbeam_channel::bounded(1);
        (Self { callback, receiver }, sender)
    }

    pub(crate) fn c_fn(&self) -> Option<unsafe extern "C" fn(*mut c_void, sys::EDiscordResult)> {
        Some(callbacks::result)
    }
}

impl<F> AnyCallback for ResultCallback<F>
where
    F: FnMut(&mut Discord, DiscordResult<()>),
{
    fn is_ready(&self) -> bool {
        !self.receiver.is_empty()
    }

    fn run(&mut self, discord: &mut Discord) {
        (self.callback)(discord, self.receiver.try_recv().unwrap())
    }
}

pub(crate) struct ResultStringCallback<F>
where
    F: FnMut(&mut Discord, DiscordResult<String>),
{
    pub(crate) callback: F,
    pub(crate) receiver: Receiver<DiscordResult<String>>,
}

impl<F> ResultStringCallback<F>
where
    F: FnMut(&mut Discord, DiscordResult<String>),
{
    pub(crate) fn new(callback: F) -> (Self, Sender<DiscordResult<String>>) {
        let (sender, receiver) = crossbeam_channel::bounded(1);
        (Self { callback, receiver }, sender)
    }

    pub(crate) fn c_fn(
        &self,
    ) -> Option<unsafe extern "C" fn(*mut c_void, sys::EDiscordResult, *const i8)> {
        Some(callbacks::result_string)
    }
}

impl<F> AnyCallback for ResultStringCallback<F>
where
    F: FnMut(&mut Discord, DiscordResult<String>),
{
    fn is_ready(&self) -> bool {
        !self.receiver.is_empty()
    }

    fn run(&mut self, discord: &mut Discord) {
        (self.callback)(discord, self.receiver.try_recv().unwrap())
    }
}

pub(crate) struct ResultBytesCallback<F>
where
    F: FnMut(&mut Discord, DiscordResult<Vec<u8>>),
{
    pub(crate) callback: F,
    pub(crate) receiver: Receiver<DiscordResult<Vec<u8>>>,
}

impl<F> ResultBytesCallback<F>
where
    F: FnMut(&mut Discord, DiscordResult<Vec<u8>>),
{
    pub(crate) fn new(callback: F) -> (Self, Sender<DiscordResult<Vec<u8>>>) {
        let (sender, receiver) = crossbeam_channel::bounded(1);
        (Self { callback, receiver }, sender)
    }

    pub(crate) fn c_fn(
        &self,
    ) -> Option<unsafe extern "C" fn(*mut c_void, sys::EDiscordResult, *mut u8, u32)> {
        Some(callbacks::result_bytes)
    }
}

impl<F> AnyCallback for ResultBytesCallback<F>
where
    F: FnMut(&mut Discord, DiscordResult<Vec<u8>>),
{
    fn is_ready(&self) -> bool {
        !self.receiver.is_empty()
    }

    fn run(&mut self, discord: &mut Discord) {
        (self.callback)(discord, self.receiver.try_recv().unwrap())
    }
}

pub(crate) struct ResultFromCallback<F, S, E>
where
    F: FnMut(&mut Discord, DiscordResult<E>),
    S: Into<E>,
{
    pub(crate) callback: F,
    pub(crate) receiver: Receiver<DiscordResult<E>>,
    _marker: PhantomData<S>,
}

impl<F, S, E> ResultFromCallback<F, S, E>
where
    F: FnMut(&mut Discord, DiscordResult<E>),
    S: Into<E>,
{
    pub(crate) fn new(callback: F) -> (Self, Sender<DiscordResult<E>>) {
        let (sender, receiver) = crossbeam_channel::bounded(1);
        (
            Self {
                callback,
                receiver,
                _marker: PhantomData,
            },
            sender,
        )
    }

    pub(crate) fn c_fn(&self) -> Option<unsafe extern "C" fn(*mut c_void, sys::EDiscordResult, S)> {
        Some(callbacks::result_from::<S, E>)
    }
}

impl<F, S, E> AnyCallback for ResultFromCallback<F, S, E>
where
    F: FnMut(&mut Discord, DiscordResult<E>),
    S: Into<E>,
{
    fn is_ready(&self) -> bool {
        !self.receiver.is_empty()
    }

    fn run(&mut self, discord: &mut Discord) {
        (self.callback)(discord, self.receiver.try_recv().unwrap())
    }
}

pub(crate) struct ResultFromPtrCallback<F, S, E>
where
    F: FnMut(&mut Discord, DiscordResult<E>),
    S: Into<E> + Sized,
{
    pub(crate) callback: F,
    pub(crate) receiver: Receiver<DiscordResult<E>>,
    _marker: PhantomData<S>,
}

impl<F, S, E> ResultFromPtrCallback<F, S, E>
where
    F: FnMut(&mut Discord, DiscordResult<E>),
    S: Into<E> + Copy,
{
    pub(crate) fn new(callback: F) -> (Self, Sender<DiscordResult<E>>) {
        let (sender, receiver) = crossbeam_channel::bounded(1);
        (
            Self {
                callback,
                receiver,
                _marker: PhantomData,
            },
            sender,
        )
    }

    pub(crate) fn c_fn(
        &self,
    ) -> Option<unsafe extern "C" fn(*mut c_void, sys::EDiscordResult, *mut S)> {
        Some(callbacks::result_from_ptr::<S, E>)
    }
}

impl<F, S, E> AnyCallback for ResultFromPtrCallback<F, S, E>
where
    F: FnMut(&mut Discord, DiscordResult<E>),
    S: Into<E> + Sized,
{
    fn is_ready(&self) -> bool {
        !self.receiver.is_empty()
    }

    fn run(&mut self, discord: &mut Discord) {
        (self.callback)(discord, self.receiver.try_recv().unwrap())
    }
}