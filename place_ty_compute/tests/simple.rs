use std::{
    collections::{BTreeMap, HashMap},
    sync::{Mutex, Once},
};

use place_ty_compute::{Field, Local, PlaceExpr, Type, place_expr};

fn init_logging() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        tracing_subscriber::registry()
            .with(
                tracing_tree::HierarchicalLayer::new(4)
                    .with_indent_lines(true)
                    .with_ansi(true)
                    .with_writer(tracing_subscriber::fmt::TestWriter::new()),
            )
            .init();
    });
}

fn check(place: &mut PlaceExpr, desugaring: &str, expected_ty: &str) {
    init_logging();
    let undesugared = format!("{place}");
    let ty = place.compute_ty();
    println!("analzed the place expression `{undesugared}` with:");
    for ctx in place.context() {
        println!("\t{ctx}")
    }
    match ty {
        Ok(ty) => {
            let mut err = false;
            if format!("{place}") != desugaring {
                err = true;
                println!();
                println!("computed desugaring does not match the expected desugaring:");
                println!("expected: {desugaring}");
                println!("computed: {place}");
            }
            if format!("{ty}") != expected_ty {
                err = true;
                println!();
                println!("computed type does not match the expected type:");
                println!("expected: {expected_ty}");
                println!("computed: {ty}");
            }
            if err {
                panic!("desugaring or type does not match expected value");
            } else {
                println!("desugared to: `{place}: {ty}`");
            }
        }
        Err(err) => {
            println!();
            println!("error while desugaring: {err}");
            println!("partial desugaring: {place}");
            panic!("an explicit error occurred during desugaring");
        }
    }
}

fn maybe_uninit(inner: &Type) -> Type {
    static CACHE: Mutex<BTreeMap<Type, Type>> = Mutex::new(BTreeMap::new());
    let mut cache = CACHE.lock().unwrap();

    if let Some(res) = cache.get(inner) {
        return res.clone();
    }

    let maybe_uninit = Type::new(
        Some(inner.clone()),
        None,
        Some(Box::new(|ty| maybe_uninit(&ty))),
        Some("MaybeUninit".to_string()),
        HashMap::new(),
        format!("MaybeUninit<{inner}>"),
    );
    cache.insert(inner.clone(), maybe_uninit.clone());
    maybe_uninit
}

fn shared_ref(target: &Type) -> Type {
    static CACHE: Mutex<BTreeMap<Type, Type>> = Mutex::new(BTreeMap::new());
    let mut cache = CACHE.lock().unwrap();

    if let Some(res) = cache.get(target) {
        return res.clone();
    }
    let shared_ref = Type::new(
        Some(target.clone()),
        None,
        None,
        None,
        HashMap::new(),
        format!("&{target}"),
    );
    cache.insert(target.clone(), shared_ref.clone());
    shared_ref
}

fn slice(target: &Type) -> Type {
    static CACHE: Mutex<BTreeMap<Type, Type>> = Mutex::new(BTreeMap::new());
    let mut cache = CACHE.lock().unwrap();

    if let Some(res) = cache.get(target) {
        return res.clone();
    }
    let slice = Type::new(
        None,
        Some(target.clone()),
        None,
        None,
        HashMap::new(),
        format!("[{target}]"),
    );
    cache.insert(target.clone(), slice.clone());
    slice
}

#[test]
fn deref() {
    let t = Type::new_generic("T");
    let t_ref = Type::new_with_target("&T", t.clone());
    let p = Local::new(t_ref, "p");
    let mut e = place_expr!(*p);
    check(&mut e, "*p", "T");
}

#[test]
fn shared_ref_field() {
    let u = Type::new_generic("U");
    let t = Type::new_struct("T", vec![Field::new("field", u.clone())]);
    let t_ref = shared_ref(&t);
    let p = Local::new(t_ref, "p");
    let mut e = place_expr!(p.field);
    check(&mut e, "(*p).field", "U");
}

#[test]
fn blog1() {
    let field = Type::new_generic("Field");
    let struct_ = Type::new_struct("Struct", vec![Field::new("field", field.clone())]);
    let mb_struct = maybe_uninit(&struct_);
    let p = Local::new(mb_struct, "p");
    let mut e = place_expr!(p.field);
    check(&mut e, "@%MaybeUninit (*p).field", "MaybeUninit<Field>");
}

#[test]
fn blog2() {
    let field = Type::new_generic("Field");
    let struct_ = Type::new_struct("Struct", vec![Field::new("field", field.clone())]);
    let mb_struct = maybe_uninit(&struct_);
    let mb_mb_struct = maybe_uninit(&mb_struct);
    let p = Local::new(mb_mb_struct, "p");
    let mut e = place_expr!(p.field);
    check(
        &mut e,
        "@%MaybeUninit @%MaybeUninit (**p).field",
        "MaybeUninit<MaybeUninit<Field>>",
    );
}

#[test]
fn blog3() {
    let field = Type::new_generic("Field");
    let struct_ = Type::new_struct("Struct", vec![Field::new("field", field.clone())]);
    let mb_struct = maybe_uninit(&struct_);
    let ty = shared_ref(&shared_ref(&shared_ref(&mb_struct)));
    let p = Local::new(ty, "p");
    let mut e = place_expr!(p.field);
    check(&mut e, "@%MaybeUninit (****p).field", "MaybeUninit<Field>");
}

#[test]
fn blog4() {
    let field = Type::new_generic("Field");
    let struct_ = Type::new_struct("Struct", vec![Field::new("field", field.clone())]);
    let ty = maybe_uninit(&shared_ref(&struct_));
    let p = Local::new(ty, "p");
    let mut e = place_expr!(p.field);
    check(&mut e, "(**p).field", "Field");
}

#[test]
fn blog5() {
    let u8 = Type::new_generic("u8");
    let ty = maybe_uninit(&slice(&u8));
    let p = Local::new(ty, "p");
    let mut e = place_expr!(p[42]);
    check(&mut e, "@%MaybeUninit (*p)[42]", "MaybeUninit<u8>");
}

#[test]
fn multi_field_auto_deref() {
    let z = Type::new_generic("Z");
    let y = Type::new_struct("Y", [Field::new("z", shared_ref(&z))]);
    let x = Type::new_struct("X", [Field::new("y", shared_ref(&y))]);
    let e = Type::new_struct("E", [Field::new("x", shared_ref(&x))]);
    let p = Local::new(shared_ref(&e), "p");
    let mut e = place_expr!(*p.x.y.z);
    check(&mut e, "*(*(*(*p).x).y).z", "Z");
}

#[test]
fn multi_wrapper() {
    let z = Type::new_generic("Z");
    let y = Type::new_struct("Y", [Field::new("z", maybe_uninit(&z))]);
    let x = Type::new_struct("X", [Field::new("y", maybe_uninit(&y))]);
    let e = Type::new_struct("E", [Field::new("x", maybe_uninit(&x))]);
    let p = Local::new(shared_ref(&e), "p");
    let mut e = place_expr!(p.x.y.z);
    check(
        &mut e,
        "@%MaybeUninit @%MaybeUninit (*(*(*p).x).y).z",
        "MaybeUninit<MaybeUninit<MaybeUninit<Z>>>",
    );
}

#[test]
fn multi_wrapper2() {
    let z = Type::new_generic("Z");
    let mbz = maybe_uninit(&z);
    let mb2z = maybe_uninit(&mbz);
    let mb3z = maybe_uninit(&mb2z);
    let y = Type::new_struct("Y", [Field::new("z", mbz.clone())]);
    let x = Type::new_struct("X", [Field::new("y", maybe_uninit(&y))]);
    let e = Type::new_struct("E", [Field::new("x", maybe_uninit(&x))]);
    let p = Local::new(shared_ref(&e), "p");
    let mut e = place_expr!(@%mb3z @%mb2z @%mbz *(*(*(*p).x).y).z);
    check(
        &mut e,
        "@%MaybeUninit @%MaybeUninit @%MaybeUninit *(*(*(*p).x).y).z",
        "MaybeUninit<MaybeUninit<MaybeUninit<Z>>>",
    );
}

#[test]
fn unexpected_maybe_uninit() {
    let field = Type::new_generic("Field");
    let struct_ = Type::new_struct("Struct", vec![Field::new("field", field.clone())]);
    let ty = maybe_uninit(&shared_ref(&maybe_uninit(&struct_)));
    let p = Local::new(ty, "p");
    let mut e = place_expr!(p.field);
    check(
        &mut e,
        // This is intended behavior, but most likely surprising.
        //
        // It is not the job of the place expression type computation to check whether the
        // operation is valid or not. Using this resulting place expression anywhere will not work,
        // since it includes a deref through a `MaybeUninit`. This is governed by the `PlaceDeref`
        // trait, which is not implemented for `MaybeUninit`. Thus this will always lead to an
        // error.
        "@%MaybeUninit (***p).field",
        "MaybeUninit<Field>",
    );
}
