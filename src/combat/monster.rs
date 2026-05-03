use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MonsterType {
    Slime,
    Skeleton,
    Bat,
    Golem,
    Boss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterState {
    Idle,
    Alert,
    Attack,
}

#[derive(Component, Debug)]
pub struct Monster {
    pub monster_type: MonsterType,
    pub health: f32,
    pub max_health: f32,
    pub speed: f32,
    pub damage: f32,
    pub detection_range: f32,
    pub attack_range: f32,
    pub state: MonsterState,
    pub spawn_depth_min: i32,
    pub spawn_depth_max: i32,
}

impl Monster {
    pub fn new(monster_type: MonsterType) -> Self {
        match monster_type {
            MonsterType::Slime => Monster {
                monster_type: MonsterType::Slime,
                health: 20.0,
                max_health: 20.0,
                speed: 2.0,
                damage: 5.0,
                detection_range: 10.0,
                attack_range: 2.0,
                state: MonsterState::Idle,
                spawn_depth_min: 0,
                spawn_depth_max: 64,
            },
            MonsterType::Skeleton => Monster {
                monster_type: MonsterType::Skeleton,
                health: 40.0,
                max_health: 40.0,
                speed: 4.0,
                damage: 10.0,
                detection_range: 16.0,
                attack_range: 8.0,
                state: MonsterState::Idle,
                spawn_depth_min: 32,
                spawn_depth_max: 128,
            },
            MonsterType::Bat => Monster {
                monster_type: MonsterType::Bat,
                health: 15.0,
                max_health: 15.0,
                speed: 6.0,
                damage: 3.0,
                detection_range: 12.0,
                attack_range: 2.0,
                state: MonsterState::Idle,
                spawn_depth_min: 0,
                spawn_depth_max: 255,
            },
            MonsterType::Golem => Monster {
                monster_type: MonsterType::Golem,
                health: 80.0,
                max_health: 80.0,
                speed: 1.5,
                damage: 20.0,
                detection_range: 8.0,
                attack_range: 3.0,
                state: MonsterState::Idle,
                spawn_depth_min: 64,
                spawn_depth_max: 192,
            },
            MonsterType::Boss => Monster {
                monster_type: MonsterType::Boss,
                health: 200.0,
                max_health: 200.0,
                speed: 3.0,
                damage: 30.0,
                detection_range: 20.0,
                attack_range: 5.0,
                state: MonsterState::Idle,
                spawn_depth_min: 192,
                spawn_depth_max: 255,
            },
        }
    }
}

pub fn update_monster_ai(monster: &mut Monster, monster_pos: Vec3, player_pos: Vec3) {
    let distance = monster_pos.distance(player_pos);

    monster.state = if distance <= monster.attack_range {
        MonsterState::Attack
    } else if distance <= monster.detection_range {
        MonsterState::Alert
    } else {
        MonsterState::Idle
    };
}

pub fn damage_monster(monster: &mut Monster, amount: f32) -> bool {
    monster.health -= amount;
    if monster.health <= 0.0 {
        monster.health = 0.0;
        true
    } else {
        false
    }
}

pub fn get_monsters_for_depth(depth: i32) -> Vec<MonsterType> {
    let all_types = [
        MonsterType::Slime,
        MonsterType::Skeleton,
        MonsterType::Bat,
        MonsterType::Golem,
        MonsterType::Boss,
    ];

    all_types
        .iter()
        .filter(|mt| {
            let m = Monster::new(**mt);
            depth >= m.spawn_depth_min && depth <= m.spawn_depth_max
        })
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_state_transitions() {
        let mut monster = Monster::new(MonsterType::Skeleton);
        let monster_pos = Vec3::new(0.0, 0.0, 0.0);

        // Far away → Idle
        let player_far = Vec3::new(20.0, 0.0, 0.0);
        update_monster_ai(&mut monster, monster_pos, player_far);
        assert_eq!(monster.state, MonsterState::Idle);

        // Within detection (16) but outside attack (8) → Alert
        let player_mid = Vec3::new(12.0, 0.0, 0.0);
        update_monster_ai(&mut monster, monster_pos, player_mid);
        assert_eq!(monster.state, MonsterState::Alert);

        // Within attack range (8) → Attack
        let player_close = Vec3::new(5.0, 0.0, 0.0);
        update_monster_ai(&mut monster, monster_pos, player_close);
        assert_eq!(monster.state, MonsterState::Attack);

        // Back to Idle when player moves away
        update_monster_ai(&mut monster, monster_pos, player_far);
        assert_eq!(monster.state, MonsterState::Idle);
    }

    #[test]
    fn test_monster_take_damage() {
        let mut monster = Monster::new(MonsterType::Slime);
        assert_eq!(monster.health, 20.0);

        // Partial damage — not dead
        let dead = damage_monster(&mut monster, 15.0);
        assert!(!dead);
        assert_eq!(monster.health, 5.0);

        // Lethal damage — dead
        let dead = damage_monster(&mut monster, 10.0);
        assert!(dead);
        assert_eq!(monster.health, 0.0);
    }

    #[test]
    fn test_monster_spawn_by_depth() {
        // Depth 0: Slime and Bat
        let depth_0 = get_monsters_for_depth(0);
        assert!(depth_0.contains(&MonsterType::Slime));
        assert!(depth_0.contains(&MonsterType::Bat));
        assert_eq!(depth_0.len(), 2);

        // Depth 64: Slime, Skeleton, Bat, Golem
        let depth_64 = get_monsters_for_depth(64);
        assert!(depth_64.contains(&MonsterType::Slime));
        assert!(depth_64.contains(&MonsterType::Skeleton));
        assert!(depth_64.contains(&MonsterType::Bat));
        assert!(depth_64.contains(&MonsterType::Golem));
        assert_eq!(depth_64.len(), 4);

        // Depth 200: Bat, Boss
        let depth_200 = get_monsters_for_depth(200);
        assert!(depth_200.contains(&MonsterType::Bat));
        assert!(depth_200.contains(&MonsterType::Boss));
        assert_eq!(depth_200.len(), 2);
    }
}
