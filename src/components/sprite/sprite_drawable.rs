use super::{fragment, vertex, Drawable};
use crate::{
    components::{Camera, Sprite, Trans},
    renderer_manager::Draw,
    Context, Id,
};
use std::sync::{Arc, RwLock};
use vulkano::{
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
        BufferUsage,
    },
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    memory::allocator::MemoryTypeFilter,
    padded::Padded,
    pipeline::{Pipeline, PipelineBindPoint},
};

pub struct SpriteDrawable;

impl SpriteDrawable {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl Drawable for SpriteDrawable {
    fn draw(
        &self,
        _: Id,
        s: Arc<RwLock<Sprite>>,
        t: Arc<RwLock<Trans>>,
        (_, c, ct): (Id, Arc<RwLock<Camera>>, Arc<RwLock<Trans>>),
        context: &Context,
        (_, builder, recreate_swapchain): &mut Draw,
    ) -> anyhow::Result<()> {
        let mut s = s.write().unwrap();
        let t = t.read().unwrap();
        let c = c.read().unwrap();
        let ct = ct.read().unwrap();
        let (vertex, fragment) = s.shaders.clone();

        if *recreate_swapchain {
            s.pipeline = Sprite::pipeline(context, vertex.clone(), fragment.clone())?;
        }

        builder.bind_pipeline_graphics(s.pipeline.clone())?;

        let view = {
            let layout = s.pipeline.layout().set_layouts().first().unwrap();
            let subbuffer_allocator = SubbufferAllocator::new(
                context.memory_allocator.clone(),
                SubbufferAllocatorCreateInfo {
                    buffer_usage: BufferUsage::UNIFORM_BUFFER,
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
            );
            let subbuffer = subbuffer_allocator.allocate_sized()?;

            *subbuffer.write()? = vertex::View {
                z: Padded(Sprite::calculate_z(c.end(), s.layer)),
                transform: <[[f32; 3]; 3]>::from(t.matrix()).map(Padded),
                camera_transform: <[[f32; 3]; 3]>::from(ct.matrix()).map(Padded),
                camera_proj: c.proj().into(),
            };

            PersistentDescriptorSet::new(
                &context.descriptor_set_allocator,
                layout.clone(),
                [WriteDescriptorSet::buffer(0, subbuffer)],
                [],
            )?
        };
        let texture = {
            let layout = s.pipeline.layout().set_layouts().get(1).unwrap();

            PersistentDescriptorSet::new(
                &context.descriptor_set_allocator,
                layout.clone(),
                [
                    WriteDescriptorSet::sampler(0, s.texture.sampler.clone()),
                    WriteDescriptorSet::image_view(1, s.texture.image.clone()),
                ],
                [],
            )?
        };
        let color = {
            let layout = s.pipeline.layout().set_layouts().get(2).unwrap();
            let subbuffer_allocator = SubbufferAllocator::new(
                context.memory_allocator.clone(),
                SubbufferAllocatorCreateInfo {
                    buffer_usage: BufferUsage::UNIFORM_BUFFER,
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
            );
            let subbuffer = subbuffer_allocator.allocate_sized()?;

            *subbuffer.write()? = fragment::Color {
                color: s.color.into(),
            };

            PersistentDescriptorSet::new(
                &context.descriptor_set_allocator,
                layout.clone(),
                [WriteDescriptorSet::buffer(0, subbuffer)],
                [],
            )?
        };

        builder
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                s.pipeline.layout().clone(),
                0,
                view.clone(),
            )?
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                s.pipeline.layout().clone(),
                1,
                texture.clone(),
            )?
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                s.pipeline.layout().clone(),
                2,
                color.clone(),
            )?
            .bind_vertex_buffers(0, s.shape.vertices.clone())?
            .draw(s.shape.vertices.len() as u32, 1, 0, 0)?;

        Ok(())
    }
}
