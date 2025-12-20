use std::{
    collections::{BTreeMap, HashMap},
    sync::Mutex,
};

use place_ty_compute::{Field, Local, PlaceExpr, Type, place_expr};

fn check(place: &mut PlaceExpr) {
    println!("analyzing the place expression `{place}` with:");
    let ty = place.compute_ty();
    for ctx in place.context() {
        println!("\t{ctx}")
    }
    match ty {
        Ok(ty) => {
            println!("desugared to: `{place}: {ty}`");
        }
        Err(err) => {
            println!();
            println!("error while desugaring: {err}");
            println!("partial desugaring: {place}");
            panic!();
        }
    }
}

#[test]
fn deref() {
    let t = Type::new_generic("T");
    let t_ref = Type::new_with_target("&T", t.clone());
    let p = Local::new(t_ref, "p");
    let mut e = place_expr!(*p);
    check(&mut e);
}

#[test]
fn shared_ref_field() {
    let u = Type::new_generic("U");
    let t = Type::new_struct("T", vec![Field::new("field", u.clone())]);
    let t_ref = Type::new_with_target("&T", t.clone());
    let p = Local::new(t_ref, "p");
    let mut e = place_expr!(p.field);
    check(&mut e);
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
fn blog1() {
    let field = Type::new_generic("Field");
    let struct_ = Type::new_struct("Struct", vec![Field::new("field", field.clone())]);
    let mb_struct = maybe_uninit(&struct_);
    let p = Local::new(mb_struct, "p");
    let mut e = place_expr!(p.field);
    check(&mut e);
}

#[test]
fn blog2() {
    let field = Type::new_generic("Field");
    let struct_ = Type::new_struct("Struct", vec![Field::new("field", field.clone())]);
    let mb_struct = maybe_uninit(&struct_);
    let mb_mb_struct = maybe_uninit(&mb_struct);
    let p = Local::new(mb_mb_struct, "p");
    let mut e = place_expr!(p.field);
    check(&mut e);
}

#[test]
fn blog3() {
    let field = Type::new_generic("Field");
    let struct_ = Type::new_struct("Struct", vec![Field::new("field", field.clone())]);
    let mb_struct = maybe_uninit(&struct_);
    let ty = shared_ref(&shared_ref(&shared_ref(&mb_struct)));
    let p = Local::new(ty, "p");
    let mut e = place_expr!(p.field);
    check(&mut e);
}

#[test]
fn blog4() {
    let field = Type::new_generic("Field");
    let struct_ = Type::new_struct("Struct", vec![Field::new("field", field.clone())]);
    let ty = maybe_uninit(&shared_ref(&struct_));
    let p = Local::new(ty, "p");
    let mut e = place_expr!(p.field);
    check(&mut e);
}

#[test]
fn blog5() {
    let u8 = Type::new_generic("u8");
    let ty = maybe_uninit(&slice(&u8));
    let p = Local::new(ty, "p");
    let mut e = place_expr!(p[42]);
    check(&mut e);
}
