mod as_asset;
mod tuple;
pub use as_asset::AsAssetsMut;
pub use tuple::AssetTuple;

use std::sync::{Arc, Mutex, OnceLock, Weak};

use bevy::{
    app::App,
    asset::{Asset, AssetId, AssetPath, AssetServer, Assets, Handle, StrongHandle, UntypedHandle},
    ecs::{
        resource::Resource,
        system::{Res, ResMut, StaticSystemParam, SystemParam},
    },
    prelude::{Deref, DerefMut},
    shader::{Shader, ShaderRef},
};

/// A temporary cache that keeps a set of assets alive.
#[derive(Debug, Clone, Default)]
pub struct AssetCache(Vec<UntypedHandle>);

impl AssetCache {
    pub const fn new() -> Self {
        AssetCache(Vec::new())
    }

    pub fn keep_alive<T: Asset>(&mut self, handle: &Handle<T>) {
        self.0.push(handle.clone().untyped());
    }

    pub fn keep_alive_untyped(&mut self, handle: &UntypedHandle) {
        self.0.push(handle.clone());
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

/// A resource that keeps a set of assets alive.
#[derive(Debug, Clone, Resource, Default, Deref, DerefMut)]
pub struct AssetCacheLayer<const LAYER: char = ' '>(AssetCache);

/// Asset server and a set of asset resources, joined with a cache layer.
#[derive(SystemParam)]
pub struct CachedAssetServer<'w, 's, A: AssetTuple = (), const LAYER: char = ' '> {
    pub asset_server: Res<'w, AssetServer>,
    pub cache_layer: ResMut<'w, AssetCacheLayer<LAYER>>,
    pub assets: StaticSystemParam<'w, 's, <A as AssetTuple>::Param>,
}

impl<A: AssetTuple, const L: char> CachedAssetServer<'_, '_, A, L> {
    /// Add an asset, if not in the asset tuple, add via [`AssetServer`].
    pub fn add<T: Asset>(&mut self, value: T) -> Handle<T> {
        let result = match A::add(&mut *self.assets, value) {
            Ok(handle) => handle,
            Err(value) => self.asset_server.add(value),
        };
        self.cache_layer.keep_alive(&result);
        result
    }

    pub fn load<T: Asset>(&mut self, path: impl Into<AssetPath<'static>>) -> Handle<T> {
        let result = self.asset_server.load(path);
        self.cache_layer.keep_alive(&result);
        result
    }
}

/// A static compatible lazy construction of assets.
pub struct LazyAssetCell<T: Asset> {
    mutex: Mutex<Option<Weak<StrongHandle>>>,
    create: fn() -> T,
}

impl<T: Asset> LazyAssetCell<T> {
    pub const fn new(f: fn() -> T) -> Self {
        LazyAssetCell {
            mutex: Mutex::new(None),
            create: f,
        }
    }

    /// Create this asset and hold it temporarily.
    pub fn get(&self, cx: &mut impl AsAssetsMut<T>) -> Handle<T> {
        let mut value = self.mutex.lock().unwrap();
        if let Some(handle) = &*value {
            if let Some(arc) = handle.upgrade() {
                return Handle::Strong(arc);
            }
        }
        let handle = cx.add((self.create)());
        match &handle {
            Handle::Strong(strong_handle) => *value = Some(Arc::downgrade(strong_handle)),
            Handle::Uuid(..) => (),
        }
        handle
    }

    /// Create this asset and hold it temporarily.
    pub fn try_get(&self) -> Option<Handle<T>> {
        let value = self.mutex.lock().unwrap();
        if let Some(handle) = &*value {
            if let Some(arc) = handle.upgrade() {
                return Some(Handle::Strong(arc));
            }
        }
        None
    }
}

/// A static compatible lazy construction of assets.
pub struct LazyShader {
    lock: OnceLock<Handle<Shader>>,
    file: &'static str,
    wgsl: &'static str,
}

impl LazyShader {
    pub const fn new(file: &'static str, wgsl: &'static str) -> Self {
        LazyShader {
            lock: OnceLock::new(),
            file,
            wgsl,
        }
    }

    /// Create this asset and hold it temporarily.
    pub fn shader_ref(&self) -> ShaderRef {
        if let Some(handle) = self.lock.get() {
            ShaderRef::Handle(handle.clone())
        } else {
            panic!("Shader {} not loaded.", self.file)
        }
    }

    /// Create this asset and hold it temporarily.
    pub fn get(&self) -> Handle<Shader> {
        if let Some(handle) = self.lock.get() {
            handle.clone()
        } else {
            panic!("Shader {} not loaded.", self.file)
        }
    }
}

/// A static compatible lazy construction of assets.
pub struct LazyAsset<T: Asset> {
    lock: OnceLock<Handle<T>>,
    create: fn() -> T,
}

impl<T: Asset> LazyAsset<T> {
    pub const fn new(create: fn() -> T) -> Self {
        LazyAsset {
            lock: OnceLock::new(),
            create,
        }
    }

    pub fn get(&self) -> Handle<T> {
        if let Some(handle) = self.lock.get() {
            handle.clone()
        } else {
            panic!("Handle not loaded.")
        }
    }

    pub fn id(&self) -> AssetId<T> {
        if let Some(handle) = self.lock.get() {
            handle.id()
        } else {
            panic!("Handle not loaded.")
        }
    }
}

pub trait LazyShaderExt {
    fn load_shader(&mut self, shader: &LazyShader);
    fn load_lazy_asset<T: Asset>(&mut self, shader: &LazyAsset<T>);
}

impl LazyShaderExt for App {
    fn load_shader(&mut self, shader: &LazyShader) {
        let id = self
            .world_mut()
            .resource_mut::<Assets<Shader>>()
            .add(Shader::from_wgsl(shader.wgsl, shader.file));
        let _ = shader.lock.set(id);
    }

    fn load_lazy_asset<T: Asset>(&mut self, asset: &LazyAsset<T>) {
        let id = self
            .world_mut()
            .resource_mut::<Assets<T>>()
            .add((asset.create)());
        let _ = asset.lock.set(id);
    }
}
