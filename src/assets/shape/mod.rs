pub mod vertex;

pub use vertex::Vertex;

use cgmath::{InnerSpace, Vector2, Zero};
use glium::{index::PrimitiveType, Display, IndexBuffer, VertexBuffer};
use std::rc::Rc;

pub static INDICES: [u32; 6] = [0, 1, 2, 1, 3, 2];

#[derive(Clone)]
pub struct Shape {
    pub vertices: Rc<VertexBuffer<Vertex>>,
    pub indices: Rc<IndexBuffer<u32>>,
}

impl Shape {
    pub fn new(
        display: &Display,
        vertices: &[Vertex],
        indices: &[u32],
        t: PrimitiveType,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            vertices: Rc::new(VertexBuffer::immutable(display, vertices)?),
            indices: Rc::new(IndexBuffer::immutable(display, t, indices)?),
        })
    }

    pub fn rect(display: &Display, dims: Vector2<f32>) -> anyhow::Result<Self> {
        let vertices = {
            let dims = dims / 2.0;

            [
                Vertex::new(Vector2::new(-dims.x, -dims.y), Vector2::zero()),
                Vertex::new(Vector2::new(-dims.x, dims.y), Vector2::new(0.0, 1.0)),
                Vertex::new(Vector2::new(dims.x, -dims.y), Vector2::new(1.0, 0.0)),
                Vertex::new(Vector2::new(dims.x, dims.y), Vector2::new(1.0, 1.0)),
            ]
        };

        Self::new(display, &vertices, &INDICES, PrimitiveType::TrianglesList)
    }

    pub fn fan(
        display: &Display,
        center: Vector2<f32>,
        points: &[Vector2<f32>],
    ) -> anyhow::Result<Self> {
        let vertices: Vec<_> = [center]
            .into_iter()
            .chain(points.iter().cloned())
            .map(|p| Vertex::new(p, p.normalize()))
            .collect();

        Self::new(display, &vertices, &INDICES, PrimitiveType::TriangleFan)
    }
}
