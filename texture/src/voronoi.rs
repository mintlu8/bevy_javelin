use noiz_math::{Vec2, Vec3};
use noiz::{
    Noise,
    cell_noise::WorleyMode,
    cells::{OrthoGrid, Voronoi},
    lengths::LengthFunction,
    prelude::{
        EuclideanLength, FractalLayers, LayeredNoise, Normed, Octave, PerCellPointDistances,
        Persistence, WorleyLeastDistance,
    },
    rng::NoiseRng,
};

use crate::{ImageBuilder, IntoImageBuilder, NoiseImage};

pub struct VoronoiImage {
    /// Since this is supposed to tile, the grid scale has to be an integer.
    pub scale: u32,
    pub randomness: f32,
    pub seed: u32,
}

impl Default for VoronoiImage {
    fn default() -> Self {
        Self {
            scale: 5,
            randomness: 1.,
            seed: 0,
        }
    }
}

pub struct GenericVoronoiImage<M: WorleyMode, L: LengthFunction<Vec2> + LengthFunction<Vec3>> {
    pub base: VoronoiImage,
    pub mode: M,
    pub length: L,
}

pub struct LayeredVoronoiImage<W: WorleyMode, L: LengthFunction<Vec2> + LengthFunction<Vec3>> {
    pub voronoi: GenericVoronoiImage<W, L>,
    pub detail: u32,
    pub roughness: f32,
    pub lacunarity: f32,
}

impl VoronoiImage {
    pub fn new(scale: u32) -> VoronoiImage {
        VoronoiImage {
            scale,
            randomness: 1.,
            seed: 0,
        }
    }

    pub fn with_layered(
        self,
        detail: u32,
        roughness: f32,
        lacunarity: f32,
    ) -> LayeredVoronoiImage<WorleyLeastDistance, EuclideanLength> {
        LayeredVoronoiImage {
            voronoi: GenericVoronoiImage {
                base: self,
                mode: WorleyLeastDistance,
                length: EuclideanLength,
            },
            detail,
            roughness,
            lacunarity,
        }
    }

    pub fn with_mode<W: WorleyMode>(self, mode: W) -> GenericVoronoiImage<W, EuclideanLength> {
        GenericVoronoiImage {
            base: self,
            mode,
            length: EuclideanLength,
        }
    }

    pub fn with_distance_fn<L: LengthFunction<Vec2> + LengthFunction<Vec3>>(
        self,
        distance: L,
    ) -> GenericVoronoiImage<WorleyLeastDistance, L> {
        GenericVoronoiImage {
            base: self,
            mode: WorleyLeastDistance,
            length: distance,
        }
    }

    pub fn with_mode_distance_fn<M, L>(self, mode: M, distance: L) -> GenericVoronoiImage<M, L>
    where
        M: WorleyMode,
        L: LengthFunction<Vec2> + LengthFunction<Vec3>,
    {
        GenericVoronoiImage {
            base: self,
            mode,
            length: distance,
        }
    }
}

impl<W: WorleyMode, L: LengthFunction<Vec2> + LengthFunction<Vec3>> GenericVoronoiImage<W, L> {
    pub fn with_layered(
        self,
        detail: u32,
        roughness: f32,
        lacunarity: f32,
    ) -> LayeredVoronoiImage<W, L> {
        LayeredVoronoiImage {
            voronoi: self,
            detail,
            roughness,
            lacunarity,
        }
    }
}

impl IntoImageBuilder for VoronoiImage {
    fn into_image_builder(self) -> impl ImageBuilder {
        GenericVoronoiImage {
            base: self,
            mode: WorleyLeastDistance,
            length: EuclideanLength,
        }
        .into_image_builder()
    }
}

impl<W: WorleyMode, L: LengthFunction<Vec2> + LengthFunction<Vec3>> IntoImageBuilder
    for GenericVoronoiImage<W, L>
{
    fn into_image_builder(self) -> impl ImageBuilder {
        NoiseImage(Noise {
            noise: PerCellPointDistances {
                cells: Voronoi::<false, OrthoGrid<i32>> {
                    partitoner: OrthoGrid(self.base.scale as i32),
                    randomness: self.base.randomness,
                },
                length_mode: self.length,
                worley_mode: self.mode,
            },
            seed: NoiseRng(self.base.seed),
            frequency: self.base.scale as f32,
        })
    }
}

impl<W: WorleyMode, L: LengthFunction<Vec2> + LengthFunction<Vec3>> IntoImageBuilder
    for LayeredVoronoiImage<W, L>
{
    fn into_image_builder(self) -> impl ImageBuilder {
        NoiseImage(Noise {
            noise: LayeredNoise::new(
                Normed::<f32>::default(),
                Persistence(self.roughness),
                FractalLayers {
                    layer: Octave(PerCellPointDistances {
                        cells: Voronoi::<false, OrthoGrid<i32>> {
                            partitoner: OrthoGrid(self.voronoi.base.scale as i32),
                            randomness: self.voronoi.base.randomness,
                        },
                        length_mode: self.voronoi.length,
                        worley_mode: self.voronoi.mode,
                    }),
                    lacunarity: self.lacunarity,
                    amount: self.detail.max(1),
                },
            ),
            seed: NoiseRng(self.voronoi.base.seed),
            frequency: self.voronoi.base.scale as f32,
        })
    }
}
