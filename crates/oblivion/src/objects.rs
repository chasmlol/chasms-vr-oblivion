//! The adapter's one object table for the runtime: the player's weapon and shield
//! ([`crate::weapons`]), their torch ([`crate::torch`]), their map ([`crate::map`]), the limbs cut off the game's creatures
//! ([`crate::gore`]), the game's things lying about near the player ([`crate::items`]), the
//! world's moving parts, its doors and levers ([`crate::world`]), and its loose Havok bodies, its
//! hanging chains ([`crate::props`]), each owner's entries kept apart and written together. The
//! same for the regions that go with the table: where the game has its own objects
//! ([`GamePoses`]) and their convex collision ([`GameHulls`]).

use rustvr_protocol::*;

/// Who an entry belongs to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Owner {
    Weapons,
    Torch,
    Gore,
    Items,
    World,
    Props,
    Map,
}

const OWNERS: usize = 7;

/// Where an owner's entries go in the table: the player's own things first, then the limbs cut
/// off (so they're never the ones the table's size leaves out), then what lies about.
fn index(owner: Owner) -> usize {
    match owner {
        Owner::Weapons => 0,
        Owner::Torch => 1,
        Owner::Map => 2,
        Owner::Gore => 3,
        Owner::Items => 4,
        Owner::World => 5,
        Owner::Props => 6,
    }
}

/// The owners' entries (pure: no link), merged into a table.
#[derive(Default)]
pub struct Parts {
    descs: [Vec<ObjectDesc>; OWNERS],
    seq: u32,
}

impl Parts {
    /// Replaces `owner`'s entries and returns the merged table (in [`index`]'s order; at most
    /// [`MAX_OBJECTS`]), its sequence number bumped.
    pub fn set(&mut self, owner: Owner, descs: &[ObjectDesc], table: &mut ObjectTable) {
        let mine = &mut self.descs[index(owner)];
        mine.clear();
        mine.extend(descs.iter().filter(|d| d.id != 0));
        table.descs.fill(bytemuck::Zeroable::zeroed());
        let all: Vec<&ObjectDesc> = self.descs.iter().flatten().take(MAX_OBJECTS).collect();
        for (i, d) in all.iter().enumerate() {
            table.descs[i] = **d;
        }
        table.count = all.len() as u32;
        self.seq = self.seq.wrapping_add(1).max(1);
        table.seq = self.seq;
    }
}

/// Each owner's game poses and hulls, merged.
#[derive(Default)]
pub struct Regions {
    poses: [Vec<GamePose>; OWNERS],
    hulls: [Vec<GameHull>; OWNERS],
}

impl Regions {
    pub fn set_poses(&mut self, owner: Owner, poses: &[GamePose], out: &mut GamePoses) {
        self.poses[index(owner)] = poses.to_vec();
        out.count = 0;
        for (i, p) in self.poses.iter().flatten().take(MAX_OBJECTS).enumerate() {
            out.poses[i] = *p;
            out.count = i as u32 + 1;
        }
    }

    pub fn set_hulls(&mut self, owner: Owner, hulls: &[GameHull], out: &mut GameHulls) {
        self.hulls[index(owner)] = hulls.to_vec();
        out.count = 0;
        for (i, h) in self.hulls.iter().flatten().take(MAX_GAME_HULLS).enumerate() {
            out.hulls[i] = *h;
            out.count = i as u32 + 1;
        }
    }
}

struct Table {
    parts: Parts,
    table: Box<ObjectTable>,
    regions: Regions,
    poses: Box<GamePoses>,
    hulls: Box<GameHulls>,
}
static TABLE: std::sync::Mutex<Option<Table>> = std::sync::Mutex::new(None);

fn with_table<R>(f: impl FnOnce(&mut Table) -> R) -> R {
    let mut t = TABLE.lock().unwrap();
    f(t.get_or_insert_with(|| Table { parts: Parts::default(), table: bytemuck::zeroed_box(), regions: Regions::default(), poses: bytemuck::zeroed_box(), hulls: bytemuck::zeroed_box() }))
}

/// Replaces `owner`'s objects and writes the whole table to the runtime.
pub fn publish(owner: Owner, descs: &[ObjectDesc], adapter: &rustvr_adapter::Adapter) -> bool {
    with_table(|t| {
        t.parts.set(owner, descs, &mut t.table);
        adapter.write_objects(&t.table)
    })
}

/// Replaces where `owner`'s own objects are (the game moves them) and writes them all.
pub fn publish_poses(owner: Owner, poses: &[GamePose], adapter: &rustvr_adapter::Adapter) -> bool {
    with_table(|t| {
        t.regions.set_poses(owner, poses, &mut t.poses);
        adapter.write_game_poses(&t.poses)
    })
}

/// Replaces `owner`'s objects' convex collision and writes them all (before the objects that use
/// them are published).
pub fn publish_hulls(owner: Owner, hulls: &[GameHull], adapter: &rustvr_adapter::Adapter) -> bool {
    with_table(|t| {
        t.regions.set_hulls(owner, hulls, &mut t.hulls);
        adapter.write_game_hulls(&t.hulls)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn desc(id: u32) -> ObjectDesc {
        let mut d: ObjectDesc = bytemuck::Zeroable::zeroed();
        d.id = id;
        d
    }

    #[test]
    fn each_owner_replaces_only_its_own_entries() {
        let mut p = Parts::default();
        let mut t: Box<ObjectTable> = bytemuck::zeroed_box();
        p.set(Owner::Weapons, &[desc(1), desc(2)], &mut t);
        p.set(Owner::Items, &[desc(10), desc(11), desc(12)], &mut t);
        let ids = |t: &ObjectTable| t.descs[..t.count as usize].iter().map(|d| d.id).collect::<Vec<_>>();
        assert_eq!(ids(&t), [1, 2, 10, 11, 12]);
        let seq = t.seq;
        p.set(Owner::Weapons, &[desc(3)], &mut t);
        assert_eq!(ids(&t), [3, 10, 11, 12], "the items stay");
        assert!(t.seq != seq);
        p.set(Owner::Items, &[], &mut t);
        assert_eq!(ids(&t), [3]);
    }
}
