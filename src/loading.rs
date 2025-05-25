//! Standard [`ProjectileBundle`](crate::ProjectileBundle) items that loads assets.

use bevy::{
    asset::{AssetPath, AssetServer, Assets},
    ecs::{bundle::Bundle, world::FilteredResourcesMut},
    pbr::{Material, MeshMaterial3d},
    render::mesh::{Mesh, Mesh2d, Mesh3d},
    sprite::{Material2d, MeshMaterial2d},
};

use crate::BundleOrAsset;

/// Add a [`Mesh`] as [`Mesh2d`].
pub struct AddMesh2(pub Mesh);

impl BundleOrAsset for AddMesh2 {
    fn to_bundle(self, resources: &mut FilteredResourcesMut) -> impl Bundle + use<> {
        if let Ok(mut assets) = resources.get_mut::<Assets<Mesh>>() {
            return Mesh2d(assets.add(self.0));
        }
        Default::default()
    }
}

/// Add a [`Material2d`] as [`MeshMaterial2d`].
pub struct AddMat2<T: Material2d>(pub T);

impl<T> BundleOrAsset for AddMat2<T>
where
    T: Material2d,
{
    fn to_bundle(self, resources: &mut FilteredResourcesMut) -> impl Bundle + use<T> {
        if let Ok(mut assets) = resources.get_mut::<Assets<T>>() {
            return MeshMaterial2d(assets.add(self.0));
        }
        Default::default()
    }
}

/// Add a [`Mesh`] as [`Mesh3d`].
pub struct AddMesh3(pub Mesh);

impl BundleOrAsset for AddMesh3 {
    fn to_bundle(self, resources: &mut FilteredResourcesMut) -> impl Bundle + use<> {
        if let Ok(mut assets) = resources.get_mut::<Assets<Mesh>>() {
            return Mesh3d(assets.add(self.0));
        }
        Default::default()
    }
}

/// Add a [`Material`] as [`MeshMaterial3d`].
pub struct AddMat3<T: Material>(pub T);

impl<T> BundleOrAsset for AddMat3<T>
where
    T: Material,
{
    fn to_bundle(self, resources: &mut FilteredResourcesMut) -> impl Bundle + use<T> {
        if let Ok(mut assets) = resources.get_mut::<Assets<T>>() {
            return MeshMaterial3d(assets.add(self.0));
        }
        Default::default()
    }
}

/// Load via [`AssetServer`].
pub struct Load<T: Bundle, F: FnOnce(&AssetServer) -> T>(pub F);

impl<T: Bundle, F: FnOnce(&AssetServer) -> T> BundleOrAsset for Load<T, F> {
    fn to_bundle(self, resources: &mut FilteredResourcesMut) -> impl Bundle + use<T, F> {
        let assets = resources
            .get::<AssetServer>()
            .expect("Expects asset server.");
        (self.0)(&assets)
    }
}

/// Load [`Mesh2d`] via [`AssetServer`].
pub struct LoadMesh2<S: Into<AssetPath<'static>> + 'static>(pub S);

impl<S: Into<AssetPath<'static>> + 'static> BundleOrAsset for LoadMesh2<S> {
    fn to_bundle(self, resources: &mut FilteredResourcesMut) -> impl Bundle + use<S> {
        let assets = resources
            .get::<AssetServer>()
            .expect("Expects asset server.");
        Mesh2d(assets.load::<Mesh>(self.0))
    }
}

/// Load [`Mesh3d`] via [`AssetServer`].
pub struct LoadMesh3<S: Into<AssetPath<'static>> + 'static>(pub S);

impl<S: Into<AssetPath<'static>> + 'static> BundleOrAsset for LoadMesh3<S> {
    fn to_bundle(self, resources: &mut FilteredResourcesMut) -> impl Bundle + use<S> {
        let assets = resources
            .get::<AssetServer>()
            .expect("Expects asset server.");
        Mesh3d(assets.load::<Mesh>(self.0))
    }
}
