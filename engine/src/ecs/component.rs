use crate::math::Transform;
use std::any::Any;

pub trait ECSComponent: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl ECSComponent for Transform {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}



// -------------


// Game State (Main Menu, In-Game)

// Main Menu:
// Layers: Main Menu UI

// In Game:
// Layers: Game (game), UI Overlay? (Health bar, cross hair, etc.), Paused?, Settings?, etc.

// in scripting world:

// horror game example. Scary entity is chasing the player.

// player = attached_entity("player") --- must be an ECSComponent
// ghost = attached_entity("ghost") --- must be an ECSComponent
// if (player.transform.position.distance(ghost.transform.position) < 5.0)
//     dead_layer.show()
//}

// Inside of the Game layer:
// ------------
// World
//  . Mesh
// Player
//  . Mesh
//  . Transform (10, 11)
//  . RigidBody2D
//  . Children
//    > Camera (0, 3) -> (10 + 0, 11 + 3)
//    . Transform (Relative to parent)  
//    . Zoom Level / Camera Props (FOV, aspect ratio, etc)

// Need to be able to write scripts!
// Wonder what language I should use? (Haskell)