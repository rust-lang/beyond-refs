use std::{
    borrow::Borrow,
    collections::{BTreeMap, HashMap, HashSet},
    fmt::{self, Display},
    hash::Hash,
    hint::unreachable_unchecked,
    ptr,
    sync::{Arc, Mutex},
};

use tracing::{debug, info, info_span};

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Ident(String);

impl Borrow<str> for Ident {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, PartialOrd, Ord, Eq, PartialEq)]
pub struct Expr(pub String);

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone)]
pub struct Type(Arc<TypeInner>);

impl Eq for Type {}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Ord for Type {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        ptr::from_ref(self.0.as_ref())
            .addr()
            .cmp(&ptr::from_ref(other.0.as_ref()).addr())
    }
}

impl PartialOrd for Type {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Type {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (&raw const *self.0).addr().hash(state)
    }
}

struct TypeInner {
    has_place_target: Option<Type>,
    array_slice_elem: Option<Type>,
    wrapper_wrap: Option<Box<dyn Fn(Type) -> Type + Send + Sync>>,
    wrapper_name: Option<String>,
    fields: HashMap<Ident, Field>,
    name: String,
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.name.fmt(f)
    }
}

impl Type {
    pub fn new(
        has_place_target: Option<Type>,
        array_slice_elem: Option<Type>,
        wrapper_wrap: Option<Box<dyn Fn(Type) -> Type + Send + Sync>>,
        wrapper_name: Option<String>,
        fields: HashMap<Ident, Field>,
        name: String,
    ) -> Self {
        assert!(wrapper_wrap.is_some() == wrapper_name.is_some());
        Self(Arc::new(TypeInner {
            has_place_target,
            array_slice_elem,
            wrapper_wrap,
            wrapper_name,
            fields,
            name,
        }))
    }

    pub fn new_generic(name: &str) -> Self {
        Self::new(None, None, None, None, HashMap::new(), name.to_string())
    }

    pub fn new_with_target(name: &str, target: Type) -> Self {
        Self::new(
            Some(target),
            None,
            None,
            None,
            HashMap::new(),
            name.to_string(),
        )
    }

    pub fn new_struct(name: &str, fields: impl IntoIterator<Item = Field>) -> Self {
        let fields = fields.into_iter().map(|f| (f.0.name.clone(), f)).collect();
        Self::new(None, None, None, None, fields, name.to_string())
    }

    fn get_has_place_target(&self) -> Option<Type> {
        self.0.has_place_target.clone()
    }

    fn get_array_or_slice_element(&self) -> Option<Type> {
        self.0.array_slice_elem.clone()
    }

    fn wrap_type(&self, compute_ty: Type) -> Option<Type> {
        self.0.wrapper_wrap.as_ref().map(|wrap| wrap(compute_ty))
    }

    fn get_field(&self, field: &str) -> Option<Field> {
        self.0.fields.get(field).cloned()
    }

    fn wrapper_name(&self) -> Option<&str> {
        self.0.wrapper_name.as_deref()
    }
}

#[derive(Clone)]
pub struct Field(Arc<FieldInner>);

impl Eq for Field {}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Hash for Field {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (&raw const *self.0).addr().hash(state)
    }
}

struct FieldInner {
    ty: Type,
    name: Ident,
}

impl Field {
    pub fn new(name: &str, ty: Type) -> Self {
        Self(Arc::new(FieldInner {
            ty,
            name: Ident(name.to_string()),
        }))
    }

    fn ty(&self) -> Type {
        self.0.ty.clone()
    }
}

#[derive(Clone)]
pub struct Local(Arc<LocalInner>);

impl Eq for Local {}

impl PartialEq for Local {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for Local {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (&raw const *self.0).addr().hash(state)
    }
}

impl PartialOrd for Local {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Local {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        ptr::from_ref(self.0.as_ref())
            .addr()
            .cmp(&ptr::from_ref(other.0.as_ref()).addr())
    }
}

struct LocalInner {
    ty: Type,
    name: Ident,
}

impl Local {
    pub fn new(ty: Type, name: &str) -> Self {
        Self(Arc::new(LocalInner {
            ty,
            name: Ident(name.to_string()),
        }))
    }

    pub fn ty(&self) -> Type {
        self.0.ty.clone()
    }
}

impl Display for Local {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.name.fmt(f)
    }
}

// ----------

pub struct Error {
    place: PlaceExpr,
    msg: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the type of `{}", self.place)?;
        if let Some(ty) = self.place.ty() {
            write!(f, ": {ty}")?;
        }
        write!(f, "` {}", self.msg)
    }
}

impl Error {
    pub fn new(p: &PlaceExpr, msg: &str) -> Self {
        Self {
            place: p.clone(),
            msg: msg.to_string(),
        }
    }
}

/// A place expression.
///
/// A place expression is a recursive data structure. The singular leaf node are local variables
/// (any local variable is a place expression). For the recursive cases, given a place expression
/// `p`, we can
/// - dereference it, written as `*p`,
/// - access a field, written as `p.field`, where `field` is an identifier,
/// - index into it, written as `p[expr]`, where `expr` is any expression,
/// - wrap it with a place wrapper, written as `@%Wrapper p`, where `Wrapper` is a `PlaceWrapper`.
#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
pub enum PlaceExpr {
    /// Local variable `v`.
    LocalVar(Local),
    /// Derefing a place `*p`.
    Deref(Box<PlaceExpr>),
    /// Accessing a field `p.field`, `field` can be any identifier.
    FieldAccess(Box<PlaceExpr>, String),
    /// Indexing a place `p[42]`, the index can be an arbitrary expression.
    Index(Box<PlaceExpr>, Expr),
    /// Wrapping a place, `@%Wrapper p`.
    Wrap(Box<PlaceExpr>, Type),
}

impl Display for PlaceExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaceExpr::Deref(p) => match &**p {
                PlaceExpr::FieldAccess(..)
                | PlaceExpr::Deref(..)
                | PlaceExpr::LocalVar(..)
                | PlaceExpr::Index(..) => {
                    write!(f, "*{p}")
                }
                PlaceExpr::Wrap(..) => write!(f, "*({p})"),
            },
            PlaceExpr::FieldAccess(p, field) => match &**p {
                PlaceExpr::FieldAccess(..) | PlaceExpr::LocalVar(..) | PlaceExpr::Index(..) => {
                    write!(f, "{p}.{field}")
                }
                PlaceExpr::Wrap(..) | PlaceExpr::Deref(..) => write!(f, "({p}).{field}"),
            },
            PlaceExpr::Index(p, i) => match &**p {
                PlaceExpr::Deref(..) | PlaceExpr::Wrap(..) => write!(f, "({p})[{i}]"),
                PlaceExpr::FieldAccess(..) | PlaceExpr::LocalVar(..) | PlaceExpr::Index(..) => {
                    write!(f, "{p}[{i}]")
                }
            },
            PlaceExpr::LocalVar(var) => write!(f, "{var}"),
            PlaceExpr::Wrap(p, ty) => write!(f, "@%{} {p}", ty.wrapper_name().unwrap()),
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
pub enum Context {
    Local(Local),
    Field(Type, Field),
}

impl Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Context::Local(local) => write!(f, "{}: {}", local, local.ty()),
            Context::Field(ty, field) => write!(f, "{}.{}: {}", ty, field.0.name, field.ty()),
        }
    }
}

impl PlaceExpr {
    pub fn context(&self) -> HashSet<Context> {
        fn _do(this: &PlaceExpr, ctx: &mut HashSet<Context>) {
            match this {
                PlaceExpr::Deref(p) => _do(p, ctx),
                PlaceExpr::FieldAccess(p, field) => {
                    if let Some(ty) = p.ty()
                        && let Some(field) = ty.get_field(field)
                    {
                        ctx.insert(Context::Field(ty, field));
                    }
                    _do(p, ctx);
                }
                PlaceExpr::Index(p, _) => _do(p, ctx),
                PlaceExpr::LocalVar(var) => {
                    ctx.insert(Context::Local(var.clone()));
                }
                PlaceExpr::Wrap(p, _) => _do(p, ctx),
            }
        }
        let mut locals = HashSet::new();
        _do(self, &mut locals);
        locals
    }

    fn wrap_in_place(&mut self, wrapper: Type) {
        let this: *mut Self = self;
        let b = Box::new_uninit();
        // SAFETY: `this` comes from a mutable reference and we write a value back later without
        // panicking.
        let val = unsafe { this.read() };
        let b = Box::write(b, val);
        let val = Self::Wrap(b, wrapper);
        // SAFETY: `this` comes from a mutable reference and we moved the value out before.
        unsafe { this.write(val) };
    }

    fn deref_in_place(&mut self) {
        let this: *mut Self = self;
        let b = Box::new_uninit();
        // SAFETY: `this` comes from a mutable reference and we write a value back later without
        // panicking.
        let val = unsafe { this.read() };
        let b = Box::write(b, val);
        let val = Self::Deref(b);
        // SAFETY: `this` comes from a mutable reference and we moved the value out before.
        unsafe { this.write(val) };
    }

    fn strip_wrap_then_deref(&mut self) {
        assert!(matches!(self, Self::Deref(p) if matches!(**p, Self::Wrap(..))));
        let this: *mut Self = self;
        // SAFETY: `this` comes from a mutable reference and we write a value back later without
        // panicking.
        let val = unsafe { this.read() };
        let Self::Deref(p) = val else {
            unsafe { unreachable_unchecked() }
        };
        let Self::Wrap(p, wrapper) = *p else {
            unsafe { unreachable_unchecked() }
        };
        unsafe { this.write(*p) };
        drop(wrapper);
    }

    /// Queries this place expressions' type without modifying it.
    ///
    /// After running [`Self::compute_ty`], this function returns `Some`.
    pub fn ty(&self) -> Option<Type> {
        match self {
            Self::LocalVar(local) => Some(local.ty()),
            Self::Deref(p) => p.ty()?.get_has_place_target(),
            Self::Index(p, _) => p.ty()?.get_array_or_slice_element(),
            Self::FieldAccess(p, field) => Some(p.ty()?.get_field(field)?.ty()),
            Self::Wrap(p, wrapper) => wrapper.wrap_type(p.ty()?),
        }
    }

    /// Compute the type of this place expression and desugar it in the process.
    ///
    /// This algorithm operates recursively on `self`. Note that it also inserts implicit
    /// dereference and place wrap operations as well as removes redundant ones (by modifying
    /// `self` in-place).
    ///
    /// Here is an informal explanation of the algorithm: we match on `self` and then proceed as
    /// follows:
    /// - When `self == l` where `l` is a local variable, we return the type of `l`.
    /// - When `self == *p` where `p` is another place expression, we
    ///   - compute the type of `p`,
    ///   - assert that `typeof(p)` implements `HasPlace`,
    ///   - if `p == @%Wrapper q` for a type `Wrapper` and place expression `q`, then:
    ///     - set `self = q`,
    ///     - compute the type of `q`,
    ///     - assert that `<typeof(p) as HasPlace>::Target` is the same as `typeof(q)`.
    ///   - return the type `<typeof(p) as HasPlace>::Target`.
    /// - When `self == p.field` or `self == p[i]`, then
    ///   - compute the type of `p`,
    ///   - now there are three cases:
    ///     1. `typeof(p)` has a field named `field` or can be indexed,
    ///     2. `typeof(p)` implement `HasPlace`,
    ///     3. None of the two cases above hold.
    ///
    ///     We cover them in reverse, since that makes it easier to understand. We also need a list
    ///     of types which we call `wrappers`. Note that the second case jumps back to the
    ///     beginning to the type computation of `p`.
    ///
    ///   - In the third case, we return an error that `typeof(p)` has no field named `field` or
    ///     cannot be indexed.
    ///   - In the second case, we append `typeof(p)` to our list of `wrappers`, then set `p = *p`,
    ///     and then jump to the beginning to compute the type of `p`.
    ///   - In the first case, we set `ty` to the type of the field or the element type and then
    ///     iterate backwards through the `wrapper` list:
    ///     - as long as the type implements `PlaceWrapper`, we set `self = @%Wrapper self` and
    ///       set `ty` to the result of wrapping `ty` with `Wrapper`,
    ///     - when we reach the end of the list or `Wrapper` doesn't implement `PlaceWrapper`, we
    ///       stop and return `ty`.
    pub fn compute_ty(&mut self) -> Result<Type, Error> {
        static CACHE: Mutex<BTreeMap<PlaceExpr, Type>> = Mutex::new(BTreeMap::new());
        let cache = CACHE.lock().unwrap();
        if let Some(ty) = cache.get(self) {
            return Ok(ty.clone());
        }
        drop(cache);
        let span = info_span!("computing type of", place = %self).entered();
        let res = span.in_scope(|| match self {
            Self::LocalVar(local) => {
                debug!("found local variable");
                info!("resolved `{local}: {}`", local.ty());
                Ok(local.ty())
            }
            Self::Deref(p) => {
                debug!("found deref, descending");
                let p_ty = p.compute_ty()?;
                debug!("expecting `{p_ty}: HasPlace`");
                if let Some(target) = p_ty.get_has_place_target() {
                    if let Self::Wrap(..) = &**p {
                        self.strip_wrap_then_deref();
                        let ty = self.compute_ty()?;
                        assert!(ty == target, "{ty} != {target}");
                    }
                    info!("resolved `{self}: {target}`");
                    Ok(target)
                } else {
                    Err(Error::new(p, "should implement `HasPlace`"))
                }
            }
            Self::Index(..) | Self::FieldAccess(..) => {
                debug!("found field/index access");
                let (p, field) = match self {
                    Self::Index(p, _) => (p, None),
                    Self::FieldAccess(p, field) => (p, Some(field)),
                    _ => unreachable!(),
                };
                let p = &mut **p;
                let mut wrappers: Vec<Type> = vec![];
                loop {
                    let p_ty = p.compute_ty()?;
                    if let Some(mut ty) = match field {
                        None => p_ty.get_array_or_slice_element(),
                        Some(ref field) => p_ty.get_field(field).map(|f| f.ty()),
                    } {
                        debug!("field/index found on `{p_ty}` with type `{ty}`");
                        for wrapper in wrappers.drain(..).rev() {
                            match wrapper.wrap_type(ty.clone()) {
                                Some(new_ty) => {
                                    debug!("wrapping with `{wrapper}`, result: `{new_ty}`");
                                    ty = new_ty;
                                    self.wrap_in_place(wrapper);
                                }
                                None => {
                                    debug!("cannot wrap with `{wrapper}`");
                                    break;
                                }
                            }
                        }
                        info!("resolved `{self}: {ty}`");
                        return Ok(ty);
                    }
                    if p_ty.get_has_place_target().is_none() {
                        debug!(
                            "no field/index found on `{p_ty}`, which also doesn't impl `HasPlace`"
                        );
                        return Err(Error::new(p, "should implement `HasPlace`"));
                    }
                    debug!("no field/index found on `{p_ty}`, adding a deref to `{p}`");
                    wrappers.push(p_ty);
                    p.deref_in_place();
                }
            }
            Self::Wrap(p, wrapper) => wrapper
                .wrap_type(p.compute_ty()?)
                .ok_or(Error::new(p, "should implement `PlaceWrapper`")),
        });
        if let Ok(ty) = &res {
            CACHE.lock().unwrap().insert(self.clone(), ty.clone());
        }
        res
    }
}

#[macro_export]
macro_rules! place_expr {
    (($($rest:tt)*)) => {
        $crate::place_expr!($($rest)*)
    };
    (*$($rest:tt)*) => {
        Box::new($crate::PlaceExpr::Deref($crate::place_expr!($($rest)*)))
    };
    ($p:ident) => {
        Box::new($crate::PlaceExpr::LocalVar($p.clone()))
    };
    ($p:tt . $field:ident $($rest:tt)+) => {
        $crate::place_expr!(($p . $field) $($rest)+)
    };
    ($p:tt . $field:ident) => {
        Box::new($crate::PlaceExpr::FieldAccess($crate::place_expr!($p), stringify!($field).to_string()))
    };
    ($p:tt [$i:expr] $($rest:tt)+) => {
        $crate::place_expr!(($p [$i]) $($rest)+)
    };
    ($p:tt [$i:expr]) => {
        Box::new($crate::PlaceExpr::Index($crate::place_expr!($p), $crate::Expr(stringify!($i).to_string())))
    };
    (@% $wrapper:ident $($p:tt)+) => {
        Box::new($crate::PlaceExpr::Wrap($crate::place_expr!($($p)+), $wrapper.clone()))
    };
}
