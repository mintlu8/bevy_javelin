use bevy::{ecs::system::SystemParam, prelude::*};
use std::any::TypeId;

type Item<'w, 's, T> = <<T as AssetTuple>::Param as SystemParam>::Item<'w, 's>;

pub trait AssetTuple: 'static {
    type Param: SystemParam;

    fn add<T: Asset>(param: &mut Item<'_, '_, Self>, value: T) -> Result<Handle<T>, T>;
}

impl AssetTuple for () {
    type Param = ();

    fn add<T: Asset>(_: &mut Item<'_, '_, Self>, value: T) -> Result<Handle<T>, T> {
        Err(value)
    }
}

fn transmute<A: 'static, B: 'static>(item: A) -> Result<B, A> {
    if TypeId::of::<A>() == TypeId::of::<B>() {
        // # Safety
        //
        // Safe since A and B are the same type.
        Ok(unsafe { std::ptr::read(&item as *const A as *const B) })
    } else {
        Err(item)
    }
}

fn transmute_handle<A: Asset, B: Asset>(item: Handle<A>) -> Handle<B> {
    match item {
        Handle::Strong(strong_handle) => Handle::Strong(strong_handle),
        Handle::Weak(asset_id) => Handle::Weak(asset_id.untyped().typed()),
    }
}

macro_rules! impl_asset_tuple {
    () => {};
    ($A: ident, $($T: ident,)*) => {
        impl<$A: Asset, $($T: Asset),*> AssetTuple for ($A, $($T,)*) {
            type Param = (ResMut<'static, Assets<$A>>, $(ResMut<'static, Assets<$T>>),*);

            #[allow(non_snake_case)]
            fn add<T: Asset>(res: &mut Item<'_, '_, Self>, mut value: T) -> Result<Handle<T>, T> {
                let ($A, $($T,)*) = res;
                match transmute::<T, $A>(value) {
                    Ok(v) => return Ok(transmute_handle($A.add(v))),
                    Err(v) => value = v,
                }
                $(
                    match transmute::<T, $T>(value) {
                        Ok(v) => return Ok(transmute_handle($T.add(v))),
                        Err(v) => value = v,
                    }
                )*
                Err(value)
            }
        }

        impl_asset_tuple!($($T,)*);
    };
}

impl_asset_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15,
);
