use support::{
    traits::{Imbalance, TryDrop},
    StorageMap,
};

use crate::currency::ElevatedTrait;
use crate::{result, AssetIdProvider, Saturating, Subtrait, Zero};
use rstd::mem;

/// Opaque, move-only struct with private fields that serves as a token denoting that
/// funds have been created without any equal and opposite accounting.
#[must_use]
pub struct PositiveImbalance<T: Subtrait, U: AssetIdProvider<AssetId = T::AssetId>>(
    T::Balance,
    rstd::marker::PhantomData<U>,
);
impl<T, U> PositiveImbalance<T, U>
where
    T: Subtrait,
    U: AssetIdProvider<AssetId = T::AssetId>,
{
    pub fn new(amount: T::Balance) -> Self {
        PositiveImbalance(amount, Default::default())
    }
}

/// Opaque, move-only struct with private fields that serves as a token denoting that
/// funds have been destroyed without any equal and opposite accounting.
#[must_use]
pub struct NegativeImbalance<T: Subtrait, U: AssetIdProvider<AssetId = T::AssetId>>(
    T::Balance,
    rstd::marker::PhantomData<U>,
);
impl<T, U> NegativeImbalance<T, U>
where
    T: Subtrait,
    U: AssetIdProvider<AssetId = T::AssetId>,
{
    pub fn new(amount: T::Balance) -> Self {
        NegativeImbalance(amount, Default::default())
    }
}

impl<T, U> TryDrop for PositiveImbalance<T, U>
where
    T: Subtrait,
    U: AssetIdProvider<AssetId = T::AssetId>,
{
    fn try_drop(self) -> result::Result<(), Self> {
        self.drop_zero()
    }
}

impl<T, U> Imbalance<T::Balance> for PositiveImbalance<T, U>
where
    T: Subtrait,
    U: AssetIdProvider<AssetId = T::AssetId>,
{
    type Opposite = NegativeImbalance<T, U>;

    fn zero() -> Self {
        Self::new(Zero::zero())
    }
    fn drop_zero(self) -> result::Result<(), Self> {
        if self.0.is_zero() {
            Ok(())
        } else {
            Err(self)
        }
    }
    fn split(self, amount: T::Balance) -> (Self, Self) {
        let first = self.0.min(amount);
        let second = self.0 - first;

        mem::forget(self);
        (Self::new(first), Self::new(second))
    }
    fn merge(mut self, other: Self) -> Self {
        self.0 = self.0.saturating_add(other.0);
        mem::forget(other);

        self
    }
    fn subsume(&mut self, other: Self) {
        self.0 = self.0.saturating_add(other.0);
        mem::forget(other);
    }
    fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
        let (a, b) = (self.0, other.0);
        mem::forget((self, other));

        if a >= b {
            Ok(Self::new(a - b))
        } else {
            Err(NegativeImbalance::new(b - a))
        }
    }
    fn peek(&self) -> T::Balance {
        self.0.clone()
    }
}

impl<T, U> TryDrop for NegativeImbalance<T, U>
where
    T: Subtrait,
    U: AssetIdProvider<AssetId = T::AssetId>,
{
    fn try_drop(self) -> result::Result<(), Self> {
        self.drop_zero()
    }
}

impl<T, U> Imbalance<T::Balance> for NegativeImbalance<T, U>
where
    T: Subtrait,
    U: AssetIdProvider<AssetId = T::AssetId>,
{
    type Opposite = PositiveImbalance<T, U>;

    fn zero() -> Self {
        Self::new(Zero::zero())
    }
    fn drop_zero(self) -> result::Result<(), Self> {
        if self.0.is_zero() {
            Ok(())
        } else {
            Err(self)
        }
    }
    fn split(self, amount: T::Balance) -> (Self, Self) {
        let first = self.0.min(amount);
        let second = self.0 - first;

        mem::forget(self);
        (Self::new(first), Self::new(second))
    }
    fn merge(mut self, other: Self) -> Self {
        self.0 = self.0.saturating_add(other.0);
        mem::forget(other);

        self
    }
    fn subsume(&mut self, other: Self) {
        self.0 = self.0.saturating_add(other.0);
        mem::forget(other);
    }
    fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
        let (a, b) = (self.0, other.0);
        mem::forget((self, other));

        if a >= b {
            Ok(Self::new(a - b))
        } else {
            Err(PositiveImbalance::new(b - a))
        }
    }
    fn peek(&self) -> T::Balance {
        self.0.clone()
    }
}

impl<T, U> Drop for PositiveImbalance<T, U>
where
    T: Subtrait,
    U: AssetIdProvider<AssetId = T::AssetId>,
{
    /// Basic drop handler will just square up the total issuance.
    fn drop(&mut self) {
        <super::TotalIssuance<ElevatedTrait<T>>>::mutate(&U::asset_id(), |v| {
            *v = v.saturating_add(self.0)
        });
    }
}

impl<T, U> Drop for NegativeImbalance<T, U>
where
    T: Subtrait,
    U: AssetIdProvider<AssetId = T::AssetId>,
{
    /// Basic drop handler will just square up the total issuance.
    fn drop(&mut self) {
        <super::TotalIssuance<ElevatedTrait<T>>>::mutate(&U::asset_id(), |v| {
            *v = v.saturating_sub(self.0)
        });
    }
}
