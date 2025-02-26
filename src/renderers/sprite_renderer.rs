use crate::{
    components::{Camera, Sprite, Trans},
    world::renderer_manager::{Draw, Renderer},
    Context, World,
};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct SpriteRenderer;

impl Renderer for SpriteRenderer {
    fn draw(
        &mut self,
        draw: &mut Draw,
        context: Arc<RwLock<Context>>,
        world: Arc<RwLock<World>>,
    ) -> anyhow::Result<()> {
        let res = {
            let em = world.read().em.clone();
            let em = em.read();

            em.entities()
                .find_map(|e| {
                    Some((
                        e,
                        em.get_component::<Camera>(e)?,
                        em.get_component::<Trans>(e)?,
                    ))
                })
                .map(|c| {
                    let sprites = {
                        let mut sprites: Vec<_> = em
                            .entities()
                            .filter_map(|e| {
                                Some((
                                    e,
                                    em.get_component::<Sprite>(e)?.clone(),
                                    em.get_component::<Trans>(e)?.clone(),
                                ))
                            })
                            .collect();

                        sprites.sort_by(|(_, s1, _), (_, s2, _)| {
                            s2.read().layer.cmp(&s1.read().layer)
                        });

                        sprites
                    };

                    (c, sprites)
                })
        };

        if let Some(((ce, c, ct), sprites)) = res {
            for (se, s, t) in sprites {
                let d = s.read().drawable.clone();

                d.draw(
                    (se, s.clone(), t.clone()),
                    (ce, c.clone(), ct.clone()),
                    draw,
                    context.clone(),
                    world.clone(),
                )?;
            }
        }

        Ok(())
    }
}
