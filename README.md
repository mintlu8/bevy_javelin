# bevy_javelin

Projectile and VFX system for bevy.

## Motivation

This crate serves as the spiritual successor of `berdicles`, that aims to reduce
frustration in creating VFX effects. `bevy_javelin` drives all the standard `ECS` stuff
like `Mesh3d`, `MeshMaterial3d`, `Sprite` and even spicy external stuff like `bevy_hanabi`'s `ParticleEffect`
through a simple interface.

## Core Features

* Easy Abstraction

An entire projectile system can be expressed as a trait `Projectile` or `ProjectileSpawner`
or spawned as a single component `ProjectileInstance`.

* Repackaged ECS

We provide a repackaged access to traditional ECS concepts like components, assets and materials.

* Garbage collection

We provide reference counted garbage collection to keep track of effects and despawn them when safe to do so.

## Non-goals

* Efficient particle system

One entity per particle is probably not the most efficient thing ever.
