use std::{
    io,
    marker::PhantomData,
    os::fd::RawFd,
};

use design::{
    ops::place::{
        PlaceHandle,
        ProjectPlace,
    },
    place::Subplace,
};

pub struct FileSlice<T> {
    fd: RawFd,
    value: PhantomData<T>,
}

pub struct FileSliceHandle<T> {
    fd: RawFd,
    value: PhantomData<T>,
}

impl<T> PlaceHandle for FileSliceHandle<T> {
    type Target = T;
}

fn offset_fd(fd: RawFd, offset: usize) {
    todo!("advance {fd} in the kernel by {offset} bytes")
}

unsafe impl<S> ProjectPlace<S> for FileSliceHandle<S::Source>
where
    S: Subplace,
    S::Source: Sized,
    S::Target: Sized,
{
    type Projected = FileSliceHandle<S::Target>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        let FileSliceHandle { fd, value: _ } = self;
        let (offset, ()) = subplace.offset(());
        offset_fd(fd, offset);
        FileSliceHandle { fd, value: PhantomData }
    }
}

impl<T> FileSlice<T> {
    pub fn read(&self) -> io::Result<T>
    where
        T: FromBytes,
    {
        todo!("read from fd: {}", self.fd)
    }
}

pub trait FromBytes {}

fn main() {}
