use crate::Projectile;
use bevy::ecs::{bundle::Bundle, world::FilteredResourcesMut};

/// A tuple starting with a [`Projectile`], with up to 15 [`Bundle`]s or [`BundleOrAsset`] implementors.
pub trait ProjectileBundle {
    fn into_projectile_bundle(
        self,
        resources: &mut FilteredResourcesMut,
    ) -> (impl Projectile, impl Bundle);
}

macro_rules! impl_bun {
    () => {};

    ($u: ident $(, $t: ident)*) => {
        #[allow(non_snake_case, unused)]
        impl<$u: Projectile $(, $t: BundleOrAsset)*> ProjectileBundle for ($u, $($t),*) {
            fn into_projectile_bundle(self, resources: &mut FilteredResourcesMut) -> (impl Projectile, impl Bundle) {
                let ($u, $(mut $t),*) = self;
                $(let $t = $t.to_bundle(resources);)*
                ($u, ($($t),*))
            }
        }

        impl_bun!($($t),*);
    };
}

impl<T: Projectile> ProjectileBundle for T {
    fn into_projectile_bundle(
        self,
        _: &mut FilteredResourcesMut,
    ) -> (impl Projectile, impl Bundle) {
        (self, ())
    }
}

impl_bun!(X, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);

/// Either a [`Bundle`] or a converter from asset to component.
#[allow(unused_variables)]
pub trait BundleOrAsset: Sized {
    fn to_bundle(self, resources: &mut FilteredResourcesMut) -> impl Bundle + use<Self>;
}

impl<T> BundleOrAsset for T
where
    T: Bundle,
{
    fn to_bundle(self, _: &mut FilteredResourcesMut) -> impl Bundle + use<T> {
        self
    }
}

#[cfg(test)]
mod test {
    use bevy::{render::view::Visibility, transform::components::Transform};

    use crate::{Projectile, ProjectileSpawner};

    use super::ProjectileBundle;

    struct Dummy;

    impl ProjectileSpawner for Dummy {}
    impl Projectile for Dummy {}

    fn bun0() -> impl ProjectileBundle {
        Dummy
    }

    fn bun1() -> impl ProjectileBundle {
        (Dummy, Transform::default())
    }

    fn bun2() -> impl ProjectileBundle {
        (Dummy, Transform::default(), Visibility::Hidden)
    }

    fn bun3() -> impl ProjectileBundle {
        (Dummy, (Transform::default(), Visibility::Hidden))
    }

    fn bun15() -> impl ProjectileBundle {
        (
            Dummy,
            Transform::default(),
            Transform::default(),
            Transform::default(),
            Transform::default(),
            Transform::default(),
            Transform::default(),
            Transform::default(),
            Transform::default(),
            Transform::default(),
            Transform::default(),
            Transform::default(),
            Transform::default(),
            Transform::default(),
            Transform::default(),
            Transform::default(),
        )
    }
}
