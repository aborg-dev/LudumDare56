use bevy::math::UVec2;
use bevy::prelude::Component;
use bevy::reflect::Reflect;

#[derive(Clone, Copy, Debug, Reflect, serde::Deserialize, Component)]
pub(crate) enum CreatureImage {
    Fox,
    Snake,
    Mouse,
    Duck,
}

const DISPLAYED_SIZE: f32 = 128.0;

impl CreatureImage {
    pub fn fox() -> Self {
        Self::Fox
    }

    pub fn size(&self) -> UVec2 {
        match self {
            CreatureImage::Fox => UVec2::splat(256),
            CreatureImage::Snake => UVec2::splat(256),
            CreatureImage::Mouse => UVec2::splat(256),
            CreatureImage::Duck => UVec2::splat(32),
        }
    }

    pub fn default_scale(&self) -> f32 {
        let size = self.size();
        DISPLAYED_SIZE / size.x.max(size.y) as f32
    }

    pub fn atlas_columns(&self) -> u32 {
        match self {
            CreatureImage::Fox => 2,
            CreatureImage::Snake => 2,
            CreatureImage::Mouse => 2,
            CreatureImage::Duck => 6,
        }
    }

    pub fn atlas_rows(&self) -> u32 {
        match self {
            CreatureImage::Fox => 1,
            CreatureImage::Snake => 1,
            CreatureImage::Mouse => 1,
            CreatureImage::Duck => 2,
        }
    }
}
