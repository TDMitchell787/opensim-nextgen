pub mod building;
pub mod scripting;
pub mod landscaping;
pub mod vehicles;
pub mod media;
pub mod clothing;
pub mod navigation;
pub mod estate;
pub mod economy;
pub mod social;
pub mod animation;
pub mod inventory;
pub mod npc_management;
pub mod tutorial;

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
