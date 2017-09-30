use treeflection::KeyedContextVec;

use ::player::Player;
use ::fighter::{Fighter, HurtBox, HitBox, CollisionBox, CollisionBoxRole, PowerShield};
use ::stage::Platform;

// def - player who was attacked
// atk - player who attacked

/// returns a list of hit results for each player
pub fn collision_check(players: &[Player], fighters: &KeyedContextVec<Fighter>, platforms: &[Platform]) -> Vec<Vec<CollisionResult>> {
    let mut result: Vec<Vec<CollisionResult>> = vec!();
    for _ in players {
        result.push(vec!());
    }

    'player_atk: for (player_atk_i, player_atk) in players.iter().enumerate() {
        let player_atk_xy = player_atk.bps_xy(players, fighters, platforms);
        let fighter_atk = &fighters[player_atk.fighter.as_ref()];
        for (player_def_i, player_def) in players.iter().enumerate() {
            let player_def_xy = player_def.bps_xy(players, fighters, platforms);
            if player_atk_i != player_def_i && player_atk.hitlist.iter().all(|x| *x != player_def_i) {
                let fighter_def = &fighters[player_def.fighter.as_ref()];

                let frame_atk = &player_atk.relative_frame(&fighter_atk.actions[player_atk.action as usize].frames[player_atk.frame as usize]);
                let frame_def = &player_def.relative_frame(&fighter_def.actions[player_def.action as usize].frames[player_def.frame as usize]);
                let colboxes_atk = frame_atk.get_hitboxes();

                'hitbox_atk: for colbox_atk in &colboxes_atk {
                    // TODO: break this out into a seperate function that can be called by the link checking code
                    let hitbox_atk = colbox_atk.hitbox_ref();

                    if colbox_shield_collision_check(player_atk_xy, colbox_atk, player_def_xy, player_def, fighter_def) {
                        result[player_atk_i].push(CollisionResult::HitShieldAtk {
                            hitbox: hitbox_atk.clone(),
                            power_shield: fighter_def.power_shield.clone(),
                            player_def_i
                        });
                        result[player_def_i].push(CollisionResult::HitShieldDef {
                            hitbox: hitbox_atk.clone(),
                            power_shield: fighter_def.power_shield.clone(),
                            player_atk_i
                        });
                        break 'hitbox_atk;
                    }

                    if hitbox_atk.enable_clang {
                        for colbox_def in frame_def.get_colboxes() {
                            match &colbox_def.role {
                            // TODO: How do we only run the clang handler once?
                                &CollisionBoxRole::Hit (ref hitbox_def) => {
                                    if let ColBoxCollisionResult::Hit = colbox_collision_check(player_atk_xy, colbox_atk, player_def_xy, colbox_def) {
                                        let damage_diff = hitbox_atk.damage as i64 - hitbox_def.damage as i64; // TODO: retrieve proper damage with move staling etc

                                        if damage_diff >= 9 {
                                            result[player_atk_i].push(CollisionResult::Clang { rebound: hitbox_atk.enable_rebound });
                                            result[player_def_i].push(CollisionResult::HitAtk { hitbox: hitbox_atk.clone(), player_def_i: player_def_i });
                                        }
                                        else if damage_diff <= -9 {
                                            result[player_atk_i].push(CollisionResult::HitAtk { hitbox: hitbox_atk.clone(), player_def_i: player_def_i });
                                            result[player_def_i].push(CollisionResult::Clang { rebound: hitbox_def.enable_rebound });
                                        }
                                        else {
                                            result[player_atk_i].push(CollisionResult::Clang { rebound: hitbox_atk.enable_rebound });
                                            result[player_def_i].push(CollisionResult::Clang { rebound: hitbox_def.enable_rebound });
                                        }
                                        break 'player_atk;
                                    }
                                }
                                _ => { }
                            }
                        }
                    }

                    for colbox_def in &frame_def.get_colboxes() {
                        match colbox_collision_check(player_atk_xy, colbox_atk, player_def_xy, colbox_def) {
                            ColBoxCollisionResult::Hit => {
                                match &colbox_def.role {
                                    &CollisionBoxRole::Hurt (ref hurtbox) => {
                                        result[player_atk_i].push(CollisionResult::HitAtk { hitbox: hitbox_atk.clone(), player_def_i: player_def_i });
                                        result[player_def_i].push(CollisionResult::HitDef { hitbox: hitbox_atk.clone(), hurtbox: hurtbox.clone(), player_atk_i: player_atk_i });
                                        break 'player_atk;
                                    }
                                    &CollisionBoxRole::Invincible => {
                                        result[player_atk_i].push(CollisionResult::HitAtk { hitbox: hitbox_atk.clone(), player_def_i: player_def_i });
                                        break 'player_atk;
                                    }
                                    _ => { }
                                }
                            }
                            ColBoxCollisionResult::Phantom => {
                                match &colbox_def.role {
                                    &CollisionBoxRole::Hurt (ref hurtbox) => {
                                        result[player_atk_i].push(CollisionResult::PhantomAtk (hitbox_atk.clone(), player_def_i));
                                        result[player_def_i].push(CollisionResult::PhantomDef (hitbox_atk.clone(), hurtbox.clone()));
                                        break 'player_atk;
                                    }
                                    _ => { }
                                }
                            }
                            ColBoxCollisionResult::None => { }
                        }
                    }
                }

                for colbox_atk in &colboxes_atk {
                    match &colbox_atk.role {
                        &CollisionBoxRole::Grab => {
                            for colbox_def in &frame_def.colboxes[..] {
                                if let ColBoxCollisionResult::Hit = colbox_collision_check(player_atk_xy, colbox_atk, player_def_xy, colbox_def) {
                                    result[player_atk_i].push(CollisionResult::GrabAtk (player_def_i));
                                    result[player_def_i].push(CollisionResult::GrabDef (player_atk_i));
                                    break 'player_atk;
                                }
                            }
                        }
                        _ => { }
                    }
                }

                // check colbox links
                // TODO
            }
        }
    }
    result
}

fn colbox_collision_check(player1_xy: (f32, f32), colbox1: &CollisionBox,  player2_xy: (f32, f32), colbox2: &CollisionBox) -> ColBoxCollisionResult {
    let x1 = player1_xy.0 + colbox1.point.0;
    let y1 = player1_xy.1 + colbox1.point.1;
    let r1 = colbox1.radius;

    let x2 = player2_xy.0 + colbox2.point.0;
    let y2 = player2_xy.1 + colbox2.point.1;
    let r2 = colbox2.radius;

    let check_distance = r1 + r2;
    let real_distance = ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt();

    if check_distance > real_distance {
        ColBoxCollisionResult::Hit
    }
    else if check_distance + 0.01 > real_distance { // TODO: customizable phantom value
        ColBoxCollisionResult::Phantom
    }
    else {
        ColBoxCollisionResult::None
    }
}

enum ColBoxCollisionResult {
    Hit,
    Phantom,
    None
}

fn colbox_shield_collision_check(player1_xy: (f32, f32), colbox1: &CollisionBox,  player2_xy: (f32, f32), player2: &Player, fighter2: &Fighter) -> bool {
    if let &Some(ref shield) = &fighter2.shield {
        if player2.is_shielding() {
            let x1 = player1_xy.0 + colbox1.point.0;
            let y1 = player1_xy.1 + colbox1.point.1;
            let r1 = colbox1.radius;

            let x2 = player2_xy.0 + player2.shield_offset_x + shield.offset_x;
            let y2 = player2_xy.1 + player2.shield_offset_y + shield.offset_y;
            let r2 = player2.shield_size(shield);

            let check_distance = r1 + r2;
            let real_distance = ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt();
            check_distance > real_distance
        } else {
            false
        }
    }
    else {
        false
    }
}

pub enum CollisionResult {
    PhantomDef   (HitBox, HurtBox),
    PhantomAtk   (HitBox, usize),
    HitDef       { hitbox: HitBox, hurtbox: HurtBox, player_atk_i: usize },
    HitAtk       { hitbox: HitBox, player_def_i: usize },
    HitShieldAtk { hitbox: HitBox, power_shield: Option<PowerShield>, player_def_i: usize },
    HitShieldDef { hitbox: HitBox, power_shield: Option<PowerShield>, player_atk_i: usize },
    ReflectDef   (HitBox), // TODO: add further details required for recreating projectile
    ReflectAtk   (HitBox),
    AbsorbDef    (HitBox),
    AbsorbAtk    (HitBox),
    GrabDef      (usize),
    GrabAtk      (usize),
    Clang        { rebound: bool },
}

// Thoughts on special cases
// *    when one hitbox connects to multiple hurtboxes HitDef is sent to all defenders
// *    when one hurtbox is hit by multiple hitboxes it receives HitDef from all attackers
