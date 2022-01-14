// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::entity::Entity;
use crate::player::{Player, Status};
use crate::protocol::*;
use crate::server::Server;
use crate::world::World;
use common::angle::Angle;
use common::entity::*;
use common::protocol::*;
use common::ticks::Ticks;
use common::util::level_to_score;
use game_server::context::PlayerTuple;
use glam::Vec2;
use rand::{thread_rng, Rng};
use rayon::iter::ParallelIterator;
use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;

impl CommandTrait for Spawn {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        let player = player_tuple.borrow_player();

        if player.data.status.is_alive() {
            return Err("cannot spawn while already alive");
        }

        if !self.entity_type.can_spawn_as(player.bot) {
            return Err("cannot spawn as given entity type");
        }

        // Default to spawning near the center of the world.
        let mut spawn_position = Vec2::ZERO;
        let mut spawn_radius = world.radius;

        let exclusion_zone = match &player.data.status {
            // Player is excluded from spawning too close to where another player sunk them, for
            // fairness reasons.
            Status::Dead {
                reason,
                position,
                time,
                ..
            } => {
                if reason.is_due_to_player() && time.elapsed() < Duration::from_secs(10) {
                    Some(*position)
                } else {
                    None
                }
            }
            _ => None,
        };

        if player.team_id.is_some() || player.invitation.is_some() {
            // TODO: Inefficient to scan all entities; only need to scan all players. Unfortunately,
            // that data is not available here, currently.
            if let Some((_, team_boat)) = world.entities.par_iter().find_any(|(_, entity)| {
                let data = entity.data();
                if data.kind == EntityKind::Boat
                    && ((player.team_id.is_some()
                        && entity.borrow_player().team_id == player.team_id)
                        || (player.invitation.is_some()
                            && entity.borrow_player().player_id
                                == player.invitation.as_ref().unwrap().player_id))
                {
                    if let Some(exclusion_zone) = exclusion_zone {
                        if entity.transform.position.distance_squared(exclusion_zone)
                            < 1250f32.powi(2)
                        {
                            // Continue.
                            return false;
                        }
                    }

                    return true;
                }
                false
            }) {
                spawn_position = team_boat.transform.position;
                spawn_radius = team_boat.data().radius + 100.0;
            }
        }

        drop(player);

        let mut boat = Entity::new(self.entity_type, Some(Arc::clone(player_tuple)));
        boat.transform.position = spawn_position;
        if world.spawn_here_or_nearby(boat, spawn_radius, exclusion_zone) {
            Ok(())
        } else {
            Err("failed to find enough space to spawn")
        }
    }
}

impl CommandTrait for Control {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        let mut player = player_tuple.borrow_player_mut();

        // Pre-borrow.
        let world_radius = world.radius;

        return if let Status::Alive {
            entity_index,
            aim_target,
            ..
        } = &mut player.data.status
        {
            let entity = &mut world.entities[*entity_index];

            // Movement
            if let Some(guidance) = self.guidance {
                entity.guidance = guidance;
            }
            *aim_target = if let Some(mut aim_target) = self.aim_target {
                sanitize_floats(aim_target.as_mut(), -world_radius * 2.0..world_radius * 2.0)?;
                Some(
                    (aim_target - entity.transform.position)
                        .clamp_length_max(entity.data().sensors.max_range())
                        + entity.transform.position,
                )
            } else {
                None
            };
            let extension = entity.extension_mut();
            extension.set_active(self.active);
            if let Some(altitude_target) = self.altitude_target {
                extension.altitude_target = altitude_target;
            }

            drop(player);

            if let Some(fire) = &self.fire {
                fire.apply(world, player_tuple)?;
            }

            if let Some(pay) = &self.pay {
                pay.apply(world, player_tuple)?;
            }

            if let Some(hint) = &self.hint {
                hint.apply(world, player_tuple)?;
            }

            Ok(())
        } else {
            Err("cannot control while not alive")
        };
    }
}

impl CommandTrait for Fire {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        let player = player_tuple.borrow_player();

        return if let Status::Alive {
            entity_index,
            aim_target,
            ..
        } = player.data.status
        {
            // Prevents limited armaments from being invalidated since all limited armaments are destroyed on upgrade.
            if player.data.flags.upgraded {
                return Err("cannot fire right after upgrading");
            }

            let entity = &mut world.entities[entity_index];

            let data = entity.data();

            let index = self.armament_index as usize;
            if index >= data.armaments.len() {
                return Err("armament index out of bounds");
            }

            if entity.extension().reloads[index] != Ticks::ZERO {
                return Err("armament not yet reloaded");
            }

            let armament = &data.armaments[index];
            let armament_entity_data = armament.entity_type.data();

            if entity.altitude.is_submerged() {
                // Submerged submarine
                if armament_entity_data.sub_kind == EntitySubKind::Shell
                    || armament_entity_data.sub_kind == EntitySubKind::Sam
                {
                    return Err("cannot fire provided armament while submerged");
                }
            }

            if let Some(turret_index) = armament.turret {
                let turret_angle = entity.extension().turrets[turret_index];
                let turret = &data.turrets[turret_index];

                // The aim may be outside the range but the turret must not be fired if the turret's
                // current angle is outside the range.
                if !turret.within_azimuth(turret_angle) {
                    return Err("invalid turret azimuth");
                }
            }

            let armament_transform =
                entity.transform + data.armament_transform(&entity.extension().turrets, index);

            let mut failed = false;
            if armament_entity_data.sub_kind == EntitySubKind::Depositor {
                let depositor = armament_transform.position;

                // Radius of depositor.
                const MAX_RADIUS: f32 = 60.0;

                // Max radius that will snap to MAX_RADIUS.
                const CUTOFF_RADIUS: f32 = MAX_RADIUS * 2.0;

                // Snap to valid radius.
                let mut position_target = self.position_target;
                sanitize_floats(
                    position_target.as_mut(),
                    -world.radius * 2.0..world.radius * 2.0,
                )?;
                let delta = position_target - depositor;
                if delta.length_squared() > CUTOFF_RADIUS.powi(2) {
                    return Err("outside maximum range");
                }
                let pos = depositor + delta.clamp_length_max(MAX_RADIUS);
                world.terrain.modify(pos, 60.0);
            } else {
                // Fire weapon.
                let player_arc = Arc::clone(player_tuple);

                drop(player);
                let mut armament_entity = Entity::new(armament.entity_type, Some(player_arc));

                armament_entity.transform = armament_transform;
                armament_entity.altitude = entity.altitude;

                let aim_angle = aim_target
                    .map(|aim| Angle::from(aim - armament_entity.transform.position))
                    .unwrap_or(entity.transform.direction);

                armament_entity.guidance.velocity_target = armament_entity_data.speed;
                armament_entity.guidance.direction_target = aim_angle;

                if armament.vertical {
                    // Vertically-launched armaments can be launched in any horizontal direction.
                    armament_entity.transform.direction = armament_entity.guidance.direction_target;
                }

                // Some weapons experience random deviation on launch
                let deviation = match armament_entity_data.sub_kind {
                    EntitySubKind::Rocket => 0.05,
                    EntitySubKind::Shell => 0.01,
                    _ => 0.03,
                };
                armament_entity.transform.direction += thread_rng().gen::<Angle>() * deviation;

                failed |= !world.spawn_here_or_nearby(armament_entity, 0.0, None);
            }

            if failed {
                Err("failed to fire from current location")
            } else {
                let entity = &mut world.entities[entity_index];
                entity.consume_armament(index);
                entity.extension_mut().clear_spawn_protection();
                Ok(())
            }
        } else {
            Err("cannot fire while not alive")
        };
    }
}

impl CommandTrait for Pay {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        let mut position = self.position;
        sanitize_floats(position.as_mut(), -world.radius * 2.0..world.radius * 2.0)?;

        let mut player = player_tuple.borrow_player_mut();

        return if let Status::Alive { entity_index, .. } = player.data.status {
            let entity = &world.entities[entity_index];

            if position.distance_squared(entity.transform.position)
                > entity.data().radii().end.powi(2)
            {
                return Err("position is too far away to pay");
            }

            let pay = 10; // Value of coin.
            let withdraw = pay * 2; // Payment has 50% efficiency.

            if player.data.score < level_to_score(entity.data().level) + withdraw {
                return Err("insufficient funds");
            }

            let mut payment = Entity::new(
                EntityType::Coin,
                Some(Arc::clone(entity.player.as_ref().unwrap())),
            );

            payment.transform.position = position;

            if world.spawn_here_or_nearby(payment, 1.0, None) {
                // Payment successfully spawned, withdraw funds.
                player.data.score -= withdraw;
            }

            Ok(())
        } else {
            Err("cannot pay while not alive")
        };
    }
}

impl CommandTrait for Hint {
    fn apply(
        &self,
        _: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        player_tuple.borrow_player_mut().data.hint = Hint {
            aspect: sanitize_float(self.aspect, 0.5..2.0)?,
        };
        Ok(())
    }
}

impl CommandTrait for Upgrade {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        let mut player = player_tuple.borrow_player_mut();
        let Player { status, score, .. } = &mut player.data;

        if let Status::Alive { entity_index, .. } = status {
            let entity = &mut world.entities[*entity_index];
            if !entity
                .entity_type
                .can_upgrade_to(self.entity_type, *score, player.bot)
            {
                return Err("cannot upgrade to provided entity type");
            }

            player.data.flags.upgraded = true;
            drop(player);

            entity.change_entity_type(self.entity_type, &mut world.arena);

            Ok(())
        } else {
            Err("cannot upgrade while not alive")
        }
    }
}

/// Returns an error if the float isn't finite. Otherwise, clamps it to the provided range.
fn sanitize_float(float: f32, valid: Range<f32>) -> Result<f32, &'static str> {
    if float.is_finite() {
        Ok(float.clamp(valid.start, valid.end))
    } else {
        Err("float not finite")
    }
}

/// Applies sanitize_float to each element.
fn sanitize_floats<'a, F: IntoIterator<Item = &'a mut f32>>(
    floats: F,
    valid: Range<f32>,
) -> Result<(), &'static str> {
    for float in floats {
        *float = sanitize_float(*float, valid.clone())?;
    }
    Ok(())
}