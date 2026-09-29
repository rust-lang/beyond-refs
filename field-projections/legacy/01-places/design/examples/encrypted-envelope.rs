#![feature(ptr_metadata)]

use std::marker::PhantomData;

use _01_places::*;

#[derive(Debug, Clone, Copy)]
pub struct Key(u8);

/// An encrypted `T`. This only holds the encrypted bytes; a key is required to
/// access the `T` value.
pub struct EncryptedEnvelope<T> {
    bytes: Box<[u8]>,
    // Value used to check that the key is the right one. It is not at all the
    // key of course haha.
    key_check: u8,
    phantom: PhantomData<T>,
}

pub struct EnvelopeBorrow<'a, Whole, Part> {
    env: &'a EncryptedEnvelope<Whole>,
    proj: Box<dyn Projection<Source = Whole, Target = Part>>,
}

impl<T> EncryptedEnvelope<T> {
    pub fn encrypt(x: T, key: Key) -> Self {
        unsafe {
            // Alloc a slice of zeros.
            let mut b: Box<[u8]> =
                Box::new_zeroed_slice(size_of::<T>()).assume_init();
            // Write the bytes of T.
            b.as_mut_ptr().cast::<T>().write_unaligned(x);
            // Xor all the bits with the key.
            for b in b.iter_mut() {
                *b ^= key.0;
            }
            Self {
                bytes: b,
                key_check: !key.0,
                phantom: PhantomData,
            }
        }
    }
    pub fn decrypt(&self, key: Key) -> T {
        // Check the key is the right one (otherwise we risk creating an invalid
        // `T`).
        assert_eq!(!key.0, self.key_check);
        unsafe {
            let mut b: Box<[u8]> = self.bytes.clone();
            // Xor all the bits with the key.
            for b in b.iter_mut() {
                *b ^= key.0;
            }
            b.as_ptr().cast::<T>().read_unaligned()
        }
    }

    // `T: 'static` is required for `dyn` for some reason.
    pub fn borrow(&self) -> EnvelopeBorrow<'_, T, T>
    where
        T: 'static,
    {
        EnvelopeBorrow {
            env: self,
            proj: Box::new(NoopProj::default()),
        }
    }
}

impl<'a, Whole, Part> EnvelopeBorrow<'a, Whole, Part> {
    pub fn decrypt(&self, key: Key) -> Part {
        let whole: &Whole = &self.env.decrypt(key);
        unsafe { self.proj.read(&raw const whole) }
    }
}

impl<T> HasPlace for EncryptedEnvelope<T> {
    type Target = T;
}
impl<'a, Whole, Part> HasPlace for EnvelopeBorrow<'a, Whole, Part> {
    type Target = Part;
}

unsafe impl<'a, 'b: 'a, Whole, P>
    PlaceBorrow<'a, P, EnvelopeBorrow<'a, Whole, P::Target>>
    for EnvelopeBorrow<'b, Whole, P::Source>
where
    P: Projection + ?Sized,
    Whole: 'static,
    P::Source: Sized + 'static,
    P::Target: Sized + 'static,
{
    const BORROW_KIND: BorrowKind = BorrowKind::Shared;
    unsafe fn borrow(
        ptr: *const Self,
        p: &P,
    ) -> EnvelopeBorrow<'a, Whole, P::Target> {
        let this = unsafe { &*ptr };
        EnvelopeBorrow {
            env: this.env,
            proj: Box::new(this.proj.as_sized().compose(p.as_sized())),
        }
    }
}

fn main() {
    #![allow(non_camel_case_types)]
    struct Foo {
        data: u32,
    }
    mk_field_proj!(struct data(Foo.data: u32));

    let key = Key(42);
    let env = EncryptedEnvelope::encrypt(Foo { data: 123456789u32 }, key);
    assert_eq!(env.decrypt(key).data, 123456789);

    let env_borrow = env.borrow();
    // Project to a field.
    let field_borrow: EnvelopeBorrow<'_, _, _> =
        unsafe { p!(@_ (*env_borrow).data) };
    assert_eq!(field_borrow.decrypt(key), 123456789);
}
