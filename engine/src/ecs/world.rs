use std::collections::HashMap;
 
use crate::ecs::{Entity, EntityId, Component};

struct World {
    entities: HashMap<EntityId, Entity>,
}

impl World {
    pub fn new() -> Self {
        
        World {
            entities: HashMap::new(),
        }
    }

    pub fn render() {
        // needs a camera entity
    }
}