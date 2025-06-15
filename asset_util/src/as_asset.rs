use bevy::{
    app::App,
    asset::{Asset, AssetServer, Assets, Handle},
    ecs::world::{FilteredResourcesMut, Mut, World},
};

use crate::{AssetTuple, CachedAssetServer};

pub trait AsAssetsMut<T: Asset> {
    fn add(&mut self, value: T) -> Handle<T>;
}

impl<T: Asset> AsAssetsMut<T> for Assets<T> {
    fn add(&mut self, value: T) -> Handle<T> {
        Assets::add(self, value)
    }
}

impl<T: Asset> AsAssetsMut<T> for Mut<'_, Assets<T>> {
    fn add(&mut self, value: T) -> Handle<T> {
        Assets::add(self, value)
    }
}

impl<T: Asset> AsAssetsMut<T> for FilteredResourcesMut<'_, '_> {
    fn add(&mut self, value: T) -> Handle<T> {
        if let Ok(mut assets) = self.get_mut::<Assets<T>>() {
            assets.add(value)
        } else {
            Handle::default()
        }
    }
}

impl<T: Asset> AsAssetsMut<T> for World {
    fn add(&mut self, value: T) -> Handle<T> {
        if let Some(mut assets) = self.get_resource_mut::<Assets<T>>() {
            assets.add(value)
        } else {
            Handle::default()
        }
    }
}

impl<T: Asset> AsAssetsMut<T> for App {
    fn add(&mut self, value: T) -> Handle<T> {
        if let Some(mut assets) = self.world_mut().get_resource_mut::<Assets<T>>() {
            assets.add(value)
        } else {
            Handle::default()
        }
    }
}

impl<T: Asset> AsAssetsMut<T> for AssetServer {
    fn add(&mut self, value: T) -> Handle<T> {
        AssetServer::add(self, value)
    }
}

impl<T: Asset, A: AssetTuple, const L: char> AsAssetsMut<T> for CachedAssetServer<'_, '_, A, L> {
    fn add(&mut self, value: T) -> Handle<T> {
        CachedAssetServer::add(self, value)
    }
}
