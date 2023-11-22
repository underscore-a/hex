use super::{ComponentManager, Context, EntityManager};
use crate::ecs::Control;
use std::sync::{Arc, RwLock};

pub trait System: Send + Sync + 'static {
    fn init(
        &mut self,
        _: Arc<RwLock<Context>>,
        _: (Arc<RwLock<EntityManager>>, Arc<RwLock<ComponentManager>>),
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn update(
        &mut self,
        _: Arc<RwLock<Control>>,
        _: Arc<RwLock<Context>>,
        _: (Arc<RwLock<EntityManager>>, Arc<RwLock<ComponentManager>>),
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
