use bevy::prelude::*;

use crate::combat::difficulty::apply_difficulty;
use crate::combat::monster::{Monster, MonsterType};
use crate::world::chunk::{Block, ChunkMap, Material};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BossAttackPattern {
    MeleeSlam,
    LavaBomb,
    SummonGolems,
    GroundShake,
}

impl BossAttackPattern {
    const CYCLE: [BossAttackPattern; 4] = [
        BossAttackPattern::MeleeSlam,
        BossAttackPattern::LavaBomb,
        BossAttackPattern::SummonGolems,
        BossAttackPattern::GroundShake,
    ];

    fn next(self) -> BossAttackPattern {
        let idx = Self::CYCLE.iter().position(|&p| p == self).unwrap_or(0);
        Self::CYCLE[(idx + 1) % Self::CYCLE.len()]
    }
}

#[derive(Debug, Clone)]
pub struct BossState {
    pub current_attack: BossAttackPattern,
    pub attack_timer: f32,
    pub phase: u32,
}

const ATTACK_INTERVAL: f32 = 2.0;
const MELEE_SLAM_RADIUS: i32 = 3;
const GROUND_SHAKE_INNER_RADIUS: i32 = 4;
const GROUND_SHAKE_OUTER_RADIUS: i32 = 8;

pub fn create_boss(depth: i32) -> (Monster, BossState) {
    let base_hp = 500.0;
    let base_damage = 30.0;
    let base_speed = 3.0;
    let (hp, damage, speed) = apply_difficulty(base_hp, base_damage, base_speed, depth);

    let mut monster = Monster::new(MonsterType::Boss);
    monster.health = hp;
    monster.max_health = hp;
    monster.damage = damage;
    monster.speed = speed;

    let boss_state = BossState {
        current_attack: BossAttackPattern::MeleeSlam,
        attack_timer: 0.0,
        phase: 1,
    };

    (monster, boss_state)
}

pub fn update_boss(
    boss_state: &mut BossState,
    _boss_pos: Vec3,
    _player_pos: Vec3,
    delta: f32,
) -> BossAttackPattern {
    boss_state.attack_timer += delta;

    if boss_state.attack_timer >= ATTACK_INTERVAL {
        boss_state.attack_timer = 0.0;
        let attack = boss_state.current_attack;
        boss_state.current_attack = attack.next();
        attack
    } else {
        boss_state.current_attack
    }
}

pub fn apply_boss_attack(
    pattern: BossAttackPattern,
    boss_pos: IVec3,
    chunk_map: &mut ChunkMap,
) -> Vec<IVec3> {
    match pattern {
        BossAttackPattern::MeleeSlam => {
            let mut affected = Vec::new();
            let r = MELEE_SLAM_RADIUS;
            for x in -r..=r {
                for y in -r..=r {
                    for z in -r..=r {
                        if x * x + y * y + z * z <= r * r {
                            let pos = IVec3::new(boss_pos.x + x, boss_pos.y + y, boss_pos.z + z);
                            chunk_map.set_block(pos, Block::air());
                            affected.push(pos);
                        }
                    }
                }
            }
            affected
        }
        BossAttackPattern::LavaBomb => {
            let target = IVec3::new(boss_pos.x, boss_pos.y - 1, boss_pos.z);
            chunk_map.set_block(target, Block::new(Material::Lava));
            vec![target]
        }
        BossAttackPattern::SummonGolems => {
            let offsets = [
                IVec3::new(2, 0, 0),
                IVec3::new(-2, 0, 0),
                IVec3::new(0, 0, 2),
            ];
            offsets
                .iter()
                .map(|off| IVec3::new(boss_pos.x + off.x, boss_pos.y + off.y, boss_pos.z + off.z))
                .collect()
        }
        BossAttackPattern::GroundShake => {
            let mut affected = Vec::new();
            for x in -GROUND_SHAKE_OUTER_RADIUS..=GROUND_SHAKE_OUTER_RADIUS {
                for z in -GROUND_SHAKE_OUTER_RADIUS..=GROUND_SHAKE_OUTER_RADIUS {
                    let dist2 = x * x + z * z;
                    let inner2 = GROUND_SHAKE_INNER_RADIUS * GROUND_SHAKE_INNER_RADIUS;
                    let outer2 = GROUND_SHAKE_OUTER_RADIUS * GROUND_SHAKE_OUTER_RADIUS;
                    if dist2 >= inner2 && dist2 <= outer2 {
                        for y in -1..=1 {
                            let pos = IVec3::new(boss_pos.x + x, boss_pos.y + y, boss_pos.z + z);
                            chunk_map.set_block(pos, Block::air());
                            affected.push(pos);
                        }
                    }
                }
            }
            affected
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::monster::damage_monster;

    #[test]
    fn test_boss_creation() {
        let (monster, boss_state) = create_boss(200);
        assert_eq!(monster.monster_type, MonsterType::Boss);
        assert!(monster.health >= 500.0);
        assert_eq!(boss_state.current_attack, BossAttackPattern::MeleeSlam);
        assert_eq!(boss_state.attack_timer, 0.0);
        assert_eq!(boss_state.phase, 1);
    }

    #[test]
    fn test_boss_attack_cycle() {
        let mut boss_state = BossState {
            current_attack: BossAttackPattern::MeleeSlam,
            attack_timer: 0.0,
            phase: 1,
        };
        let boss_pos = Vec3::ZERO;
        let player_pos = Vec3::ZERO;

        let attack = update_boss(&mut boss_state, boss_pos, player_pos, 2.5);
        assert_eq!(attack, BossAttackPattern::MeleeSlam);
        assert_eq!(boss_state.current_attack, BossAttackPattern::LavaBomb);

        let attack = update_boss(&mut boss_state, boss_pos, player_pos, 2.5);
        assert_eq!(attack, BossAttackPattern::LavaBomb);
        assert_eq!(boss_state.current_attack, BossAttackPattern::SummonGolems);

        let attack = update_boss(&mut boss_state, boss_pos, player_pos, 2.5);
        assert_eq!(attack, BossAttackPattern::SummonGolems);
        assert_eq!(boss_state.current_attack, BossAttackPattern::GroundShake);

        let attack = update_boss(&mut boss_state, boss_pos, player_pos, 2.5);
        assert_eq!(attack, BossAttackPattern::GroundShake);
        assert_eq!(boss_state.current_attack, BossAttackPattern::MeleeSlam);
    }

    #[test]
    fn test_boss_takes_damage() {
        let (mut monster, _) = create_boss(100);
        let initial_health = monster.health;
        let dead = damage_monster(&mut monster, 50.0);
        assert!(!dead);
        assert!(monster.health < initial_health);

        let remaining = monster.health + 1.0;
        let dead = damage_monster(&mut monster, remaining);
        assert!(dead);
        assert_eq!(monster.health, 0.0);
    }

    #[test]
    fn test_melee_slam_destroys_blocks() {
        use crate::world::chunk::Chunk;
        let mut map = ChunkMap::default();
        let mut chunk = Chunk::new(IVec3::ZERO);
        let center = IVec3::new(8, 8, 8);
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    chunk.set_block(x, y, z, Block::new(Material::Stone));
                }
            }
        }
        map.insert_chunk(chunk);
        let affected = apply_boss_attack(BossAttackPattern::MeleeSlam, center, &mut map);
        assert!(!affected.is_empty());
        assert_eq!(map.get_block(center), Material::Air);
    }

    #[test]
    fn test_lava_bomb_places_lava() {
        let mut map = ChunkMap::default();
        let boss_pos = IVec3::new(8, 8, 8);
        let affected = apply_boss_attack(BossAttackPattern::LavaBomb, boss_pos, &mut map);
        let target = IVec3::new(8, 7, 8);
        assert_eq!(affected.len(), 1);
        assert_eq!(affected[0], target);
    }

    #[test]
    fn test_summon_golems_returns_spawn_positions() {
        let mut map = ChunkMap::default();
        let boss_pos = IVec3::new(10, 5, 10);
        let spawns = apply_boss_attack(BossAttackPattern::SummonGolems, boss_pos, &mut map);
        assert_eq!(spawns.len(), 3);
        assert!(spawns.contains(&IVec3::new(12, 5, 10)));
        assert!(spawns.contains(&IVec3::new(8, 5, 10)));
        assert!(spawns.contains(&IVec3::new(10, 5, 12)));
    }

    #[test]
    fn test_ground_shake_ring_pattern() {
        use crate::world::chunk::Chunk;
        let mut map = ChunkMap::default();
        for cx in 0..3 {
            for cy in 0..2 {
                for cz in 0..3 {
                    let mut chunk = Chunk::new(IVec3::new(cx, cy, cz));
                    for x in 0..16 {
                        for y in 0..16 {
                            for z in 0..16 {
                                chunk.set_block(x, y, z, Block::new(Material::Stone));
                            }
                        }
                    }
                    map.insert_chunk(chunk);
                }
            }
        }
        let boss_pos = IVec3::new(16, 8, 16);
        let affected = apply_boss_attack(BossAttackPattern::GroundShake, boss_pos, &mut map);
        assert_eq!(map.get_block(boss_pos), Material::Stone);
        assert!(!affected.is_empty());
    }
}
