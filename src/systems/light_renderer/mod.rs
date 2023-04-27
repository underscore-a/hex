use crate::{
    assets::Shader,
    components::{Camera, Light, Model, Transform},
    ecs::{system_manager::System, ComponentManager, EntityManager, Ev, Scene},
    math::Vec2d,
};
use glium::{
    draw_parameters::{BackfaceCullingMode, Blend, DepthTest},
    framebuffer::SimpleFrameBuffer,
    texture::{DepthTexture2d, Texture2d},
    uniform,
    uniforms::{MagnifySamplerFilter, Sampler, SamplerBehavior},
    Depth, Display, DrawParameters, Surface,
};

pub struct LightRenderer<'a> {
    pub lighting_draw_parameters: DrawParameters<'a>,
    pub shadow_draw_parameters: DrawParameters<'a>,
    pub shadow_shader: Shader,
    pub lighting_shader: Shader,
    pub shadow_dims: (u32, u32),
    pub filter: MagnifySamplerFilter,
    pub sampler_behavior: SamplerBehavior,
}

impl<'a> LightRenderer<'a> {
    pub fn new(
        display: &Display,
        sampler_behavior: SamplerBehavior,
        filter: MagnifySamplerFilter,
        shadow_dims: (u32, u32),
    ) -> anyhow::Result<Self> {
        Ok(Self {
            lighting_draw_parameters: DrawParameters {
                depth: Depth {
                    test: DepthTest::IfLessOrEqual,
                    write: true,
                    ..Default::default()
                },
                blend: Blend::alpha_blending(),
                backface_culling: BackfaceCullingMode::CullClockwise,
                ..Default::default()
            },
            shadow_draw_parameters: DrawParameters {
                depth: Depth {
                    test: DepthTest::IfLessOrEqual,
                    write: true,
                    ..Default::default()
                },
                backface_culling: BackfaceCullingMode::CullCounterClockwise,
                ..Default::default()
            },
            shadow_shader: Shader::new(
                display,
                include_str!("shadow/vertex.glsl"),
                include_str!("shadow/fragment.glsl"),
                None,
            )?,
            lighting_shader: Shader::new(
                display,
                include_str!("lighting/vertex.glsl"),
                include_str!("lighting/fragment.glsl"),
                None,
            )?,
            sampler_behavior,
            filter,
            shadow_dims,
        })
    }
}

impl<'a> System<'a> for LightRenderer<'a> {
    fn update(
        &mut self,
        event: &mut Ev,
        scene: &mut Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let Ev::Draw((_, target)) = event {
            if let Some((c, ct)) = em.entities.keys().cloned().find_map(|e| {
                Some((
                    cm.get::<Camera>(e, em)
                        .and_then(|c| (c.active && c.main).then_some(c))?,
                    cm.get::<Transform>(e, em)
                        .and_then(|t| t.active.then_some(t))?,
                ))
            }) {
                let models = {
                    let mut models: Vec<_> = em
                        .entities
                        .keys()
                        .cloned()
                        .filter_map(|e| {
                            Some((
                                cm.get::<Model>(e, em).and_then(|s| s.active.then_some(s))?,
                                cm.get::<Transform>(e, em)
                                    .and_then(|t| t.active.then_some(t))?,
                            ))
                        })
                        .collect();

                    models.sort_by(|(_, t1), (_, t2)| {
                        (ct.position() - t1.position())
                            .magnitude()
                            .total_cmp(&(ct.position() - t2.position()).magnitude())
                    });

                    models
                };

                let (surface_width, surface_height) = target.get_dimensions();
                let buffer = Texture2d::empty(&scene.display, surface_width, surface_height)?;
                let shadow_buffer =
                    DepthTexture2d::empty(&scene.display, surface_width, surface_height)?;
                let mut shadow_target =
                    SimpleFrameBuffer::depth_only(&scene.display, &shadow_buffer)?;
                let lights = em
                    .entities
                    .keys()
                    .cloned()
                    .filter_map(|e| {
                        Some((
                            e,
                            cm.get::<Light>(e, em).and_then(|l| l.active.then_some(l))?,
                            cm.get::<Transform>(e, em)
                                .and_then(|t| t.active.then_some(t))?,
                        ))
                    })
                    .collect::<Vec<_>>();

                shadow_target.clear_depth(1.0);

                for (lc, _, lt) in lights.iter().filter_map(|(e, l, lt)| {
                    Some((
                        cm.get::<Camera>(*e, em)
                            .and_then(|c| c.active.then_some(c))?,
                        l,
                        lt,
                    ))
                }) {
                    for (m, t) in &models {
                        let (mesh, _, _) = &*m.data;
                        let (v, i) = &*mesh.buffer;
                        let u = uniform! {
                            transform: t.matrix().0,
                            light_proj: lc.proj().0,
                            light_transform: lt.matrix().0,
                        };

                        shadow_target.draw(
                            v,
                            i.source(),
                            &self.shadow_shader.program,
                            &u,
                            &self.shadow_draw_parameters,
                        )?;
                    }
                }

                for (_, l, lt) in &lights {
                    for (m, t) in &models {
                        target.fill(&buffer.as_surface(), self.filter);

                        let (shadow_width, shadow_height) = self.shadow_dims;
                        let (mesh, ma, _) = &*m.data;
                        let (v, i) = &*mesh.buffer;
                        let u = uniform! {
                            transform: t.matrix().0,
                            camera_transform: ct.matrix().0,
                            camera_proj: c.proj().0,
                            buffer: Sampler(&buffer, self.sampler_behavior),
                            shadow_buffer: Sampler(&shadow_buffer, self.sampler_behavior),
                            camera_position: ct.position().0,
                            light_color: l.color.0,
                            light_position: lt.position().0,
                            screen_dims: Vec2d::new(surface_width as f32, surface_height as f32).0,
                            shadow_dims: Vec2d::new(shadow_width as f32, shadow_height as f32).0,
                            light_strength: l.strength,
                            ambient_strength: ma.ambient,
                            diffuse_strength: ma.diffuse,
                            specular_strength: ma.specular,
                            reflect_strength: ma.reflect,
                            bias: ma.bias,
                        };

                        target.draw(
                            v,
                            i.source(),
                            &self.lighting_shader.program,
                            &u,
                            &self.lighting_draw_parameters,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}
