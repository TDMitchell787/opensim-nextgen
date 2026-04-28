pub mod animation;
pub mod building;
pub mod clothing;
pub mod economy;
pub mod estate;
pub mod inventory;
pub mod landscaping;
pub mod media;
pub mod navigation;
pub mod npc_management;
pub mod scripting;
pub mod social;
pub mod tutorial;
pub mod vehicles;

use super::skill_engine::SkillRegistry;

pub fn register_all(registry: &mut SkillRegistry) {
    building::register(registry);
    scripting::register(registry);
    landscaping::register(registry);
    vehicles::register(registry);
    media::register(registry);
    clothing::register(registry);
    navigation::register(registry);
    estate::register(registry);
    economy::register(registry);
    social::register(registry);
    animation::register(registry);
    inventory::register(registry);
    npc_management::register(registry);
    tutorial::register(registry);
}
