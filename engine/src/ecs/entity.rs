use std::collections::HashMap;
use ecs::{EntityId, Component};

struct EntityId {
    id: u32,
}

struct Entity {
    id: EntityId,
    name: String,
    children: Vec<EntityId>,
    components: HashMap<TypeId, HashMap<String, Box<dyn Component>>>,   
}

impl Entity {
    pub fn new(id: EntityId, name: &str) -> Self {
        
        Entity {
            id,
            name: name.to_string(),
            children: Vec::new(),
            components: HashMap::new(),
        }
    }
}

