use bevy::asset::{Asset, Handle};
use bevy::reflect::{std_traits::ReflectDefault, Reflect};
use bevy::render::{
    color::Color,
    mesh::MeshVertexBufferLayout,
    render_asset::RenderAssets,
    render_resource::{
        AsBindGroup, AsBindGroupShaderType, PolygonMode, RenderPipelineDescriptor, ShaderRef,
        SpecializedMeshPipelineError,
    },
    texture::Image,
};
use bevy::sprite::{
    ColorMaterialFlags, ColorMaterialUniform, Material2d, Material2dKey,
    COLOR_MATERIAL_SHADER_HANDLE,
};

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
#[reflect(Default, Debug)]
#[uniform(0, ColorMaterialUniform)]
pub struct LineColorMaterial {
    pub color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
}

impl Default for LineColorMaterial {
    fn default() -> Self {
        LineColorMaterial {
            color: Color::WHITE,
            texture: None,
        }
    }
}

impl From<Color> for LineColorMaterial {
    fn from(color: Color) -> Self {
        LineColorMaterial {
            color,
            ..Default::default()
        }
    }
}

impl From<Handle<Image>> for LineColorMaterial {
    fn from(texture: Handle<Image>) -> Self {
        LineColorMaterial {
            texture: Some(texture),
            ..Default::default()
        }
    }
}

impl AsBindGroupShaderType<ColorMaterialUniform> for LineColorMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<Image>) -> ColorMaterialUniform {
        let mut flags = ColorMaterialFlags::NONE;
        if self.texture.is_some() {
            flags |= ColorMaterialFlags::TEXTURE;
        }

        ColorMaterialUniform {
            color: self.color.as_linear_rgba_f32().into(),
            flags: flags.bits(),
        }
    }
}

impl Material2d for LineColorMaterial {
    fn fragment_shader() -> ShaderRef {
        COLOR_MATERIAL_SHADER_HANDLE.into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}
