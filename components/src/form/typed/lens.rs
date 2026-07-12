//! Static lenses into a form struct `T`.
//!
//! The `FormFields` derive stores plain fn pointers on each field const
//! (`FieldName::get`/`get_mut`, `FieldArray::get`/`get_mut`). This module
//! turns those into a composable [`Lens`] so nested paths
//! (`Order::customer.then(Customer::email)`, `Order::items.nth(2)`) read and
//! write the struct directly — no serde, no string maps.
//!
//! Every lens also carries the dot-notation path (`"items.2.qty"`) used to
//! key the string-indexed aux state (touched/errors/overlay/pristine).

use crate::field_name::{FieldArray, FieldName, FieldType};

/// A statically-composed accessor into `Root`.
///
/// `get` returns `None` when a segment is missing at runtime (unset
/// `Option`, out-of-range index). `get_mut` materializes missing segments
/// (inserts defaults) so writes always land.
pub trait Lens<Root>: Copy + 'static {
    type Value;

    fn get<'a>(&self, root: &'a Root) -> Option<&'a Self::Value>;
    fn get_mut<'a>(&self, root: &'a mut Root) -> &'a mut Self::Value;

    /// Dot-notation path keying the aux-state maps.
    fn path(&self) -> String;
    /// Human label of the leaf segment.
    fn label(&self) -> &'static str;
    /// Whether the leaf segment is required (derived from `!Option` in the
    /// struct definition).
    fn required(&self) -> bool;
    /// Schema type of the leaf segment.
    fn field_type(&self) -> FieldType;
}

/// Composition/navigation helpers available on every lens.
pub trait LensExt<Root>: Lens<Root> + Sized {
    /// Descend into a field of the current value: `order.then(Customer::email)`.
    fn then<C>(self, child: C) -> Compose<Self, C>
    where
        C: Lens<Self::Value>,
    {
        Compose {
            parent: self,
            child,
        }
    }

    /// Index into a `Vec` value: `Order::items.nth(2)`.
    fn nth<E>(self, index: usize) -> Index<Self>
    where
        Self: Lens<Root, Value = Vec<E>>,
    {
        Index {
            parent: self,
            index,
        }
    }

    /// Descend through an `Option` value, materializing a default on write.
    /// Transparent in the path (matches the old JSON flattening semantics).
    fn some<M>(self) -> Inner<Self>
    where
        Self: Lens<Root, Value = Option<M>>,
        M: Default,
    {
        Inner { parent: self }
    }
}

impl<Root, L: Lens<Root>> LensExt<Root> for L {}

impl<T: 'static, F: 'static> Lens<T> for FieldName<T, F> {
    type Value = F;

    fn get<'a>(&self, root: &'a T) -> Option<&'a F> {
        Some((self.get)(root))
    }

    fn get_mut<'a>(&self, root: &'a mut T) -> &'a mut F {
        (self.get_mut)(root)
    }

    fn path(&self) -> String {
        self.name.to_string()
    }

    fn label(&self) -> &'static str {
        self.label
    }

    fn required(&self) -> bool {
        self.required
    }

    fn field_type(&self) -> FieldType {
        self.field_type
    }
}

impl<T: 'static, F: 'static> Lens<T> for FieldArray<T, F> {
    type Value = Vec<F>;

    fn get<'a>(&self, root: &'a T) -> Option<&'a Vec<F>> {
        (self.get)(root)
    }

    fn get_mut<'a>(&self, root: &'a mut T) -> &'a mut Vec<F> {
        (self.get_mut)(root)
    }

    fn path(&self) -> String {
        self.name.to_string()
    }

    fn label(&self) -> &'static str {
        self.label
    }

    fn required(&self) -> bool {
        self.required
    }

    fn field_type(&self) -> FieldType {
        FieldType::Array
    }
}

/// `parent.child` — descends into a field of the parent's value.
#[derive(Clone, Copy, PartialEq)]
pub struct Compose<A, B> {
    parent: A,
    child: B,
}

impl<Root, A, B> Lens<Root> for Compose<A, B>
where
    A: Lens<Root>,
    B: Lens<A::Value>,
    A::Value: 'static,
{
    type Value = B::Value;

    fn get<'a>(&self, root: &'a Root) -> Option<&'a Self::Value> {
        self.parent.get(root).and_then(|mid| self.child.get(mid))
    }

    fn get_mut<'a>(&self, root: &'a mut Root) -> &'a mut Self::Value {
        self.child.get_mut(self.parent.get_mut(root))
    }

    fn path(&self) -> String {
        format!("{}.{}", self.parent.path(), self.child.path())
    }

    fn label(&self) -> &'static str {
        self.child.label()
    }

    fn required(&self) -> bool {
        self.child.required()
    }

    fn field_type(&self) -> FieldType {
        self.child.field_type()
    }
}

/// `parent.N` — indexes into a `Vec` value. Writes past the end grow the
/// `Vec` with defaults so the write always lands (mirrors the old JSON-null
/// padding).
#[derive(Clone, Copy, PartialEq)]
pub struct Index<L> {
    parent: L,
    index: usize,
}

impl<Root, L, E> Lens<Root> for Index<L>
where
    L: Lens<Root, Value = Vec<E>>,
    E: Default + 'static,
{
    type Value = E;

    fn get<'a>(&self, root: &'a Root) -> Option<&'a E> {
        self.parent.get(root)?.get(self.index)
    }

    fn get_mut<'a>(&self, root: &'a mut Root) -> &'a mut E {
        let vec = self.parent.get_mut(root);
        while vec.len() <= self.index {
            vec.push(E::default());
        }
        &mut vec[self.index]
    }

    fn path(&self) -> String {
        format!("{}.{}", self.parent.path(), self.index)
    }

    fn label(&self) -> &'static str {
        self.parent.label()
    }

    fn required(&self) -> bool {
        self.parent.required()
    }

    fn field_type(&self) -> FieldType {
        self.parent.field_type()
    }
}

/// Descends through `Option<M>`; reads short-circuit on `None`, writes
/// materialize `M::default()`. Path-transparent.
#[derive(Clone, Copy, PartialEq)]
pub struct Inner<L> {
    parent: L,
}

impl<Root, L, M> Lens<Root> for Inner<L>
where
    L: Lens<Root, Value = Option<M>>,
    M: Default + 'static,
{
    type Value = M;

    fn get<'a>(&self, root: &'a Root) -> Option<&'a M> {
        self.parent.get(root)?.as_ref()
    }

    fn get_mut<'a>(&self, root: &'a mut Root) -> &'a mut M {
        self.parent.get_mut(root).get_or_insert_with(M::default)
    }

    fn path(&self) -> String {
        self.parent.path()
    }

    fn label(&self) -> &'static str {
        self.parent.label()
    }

    fn required(&self) -> bool {
        self.parent.required()
    }

    fn field_type(&self) -> FieldType {
        self.parent.field_type()
    }
}
