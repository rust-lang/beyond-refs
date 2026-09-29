use std::ptr::Pointee;

mod non_null;
mod raw_const;
mod raw_mut;

/// Pointer metadata.
///
/// This alias is often more convenient than typing `<T as Pointee>::Metadata`. It also is nicer to
/// read, since when `T` is rather long, because reading `<MyVeryLongLooooongLoooong<Type, With, Many,
/// Generics> as Pointee>::Metadata` feels like "here's a long type, **psych** it's actually its
/// metadata that we want!".
pub type Metadata<T> = <T as Pointee>::Metadata;
