use super::entity::Entity;

pub struct EntityManager {
    entities: Vec<Entity>,
}
impl EntityManager {
    pub fn new() -> Self {
        EntityManager {
            entities: Vec::new(),
        }
    }
}
