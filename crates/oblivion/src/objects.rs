//! The adapter's one object table for the runtime: the player's weapon and shield
//! ([`crate::weapons`]) and the game's things lying about near the player ([`crate::items`]), each
//! owner's entries kept apart and written together; and the same for their convex collision (the
//! game's Havok shapes, [`publish_hulls`]).

use rustvr_protocol::*;

/// Who an entry belongs to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Owner {
    Weapons,
    Items,
}

/// The owners' entries (pure: no link), merged into a table.
#[derive(Default)]
pub struct Parts {
    weapons: Vec<ObjectDesc>,
    items: Vec<ObjectDesc>,
    seq: u32,
}

impl Parts {
    /// Replaces `owner`'s entries and returns the merged table (weapons first; at most
    /// [`MAX_OBJECTS`]), its sequence number bumped.
    pub fn set(&mut self, owner: Owner, descs: &[ObjectDesc], table: &mut ObjectTable) {
        let mine = match owner {
            Owner::Weapons => &mut self.weapons,
            Owner::Items => &mut self.items,
        };
        mine.clear();
        mine.extend(descs.iter().filter(|d| d.id != 0));
        table.descs.fill(bytemuck::Zeroable::zeroed());
        let all: Vec<&ObjectDesc> = self.weapons.iter().chain(&self.items).take(MAX_OBJECTS).collect();
        for (i, d) in all.iter().enumerate() {
            table.descs[i] = **d;
        }
        table.count = all.len() as u32;
        self.seq = self.seq.wrapping_add(1).max(1);
        table.seq = self.seq;
    }
}

struct Table {
    parts: Parts,
    table: Box<ObjectTable>,
}
static TABLE: std::sync::Mutex<Option<Table>> = std::sync::Mutex::new(None);

/// Each owner's convex parts, and the table they're written in together.
struct Hulls {
    weapons: Vec<GameHull>,
    items: Vec<GameHull>,
    table: Box<GameHulls>,
}
static HULLS: std::sync::Mutex<Option<Hulls>> = std::sync::Mutex::new(None);

/// Replaces `owner`'s convex parts and writes all of them to the runtime (the weapons' first).
pub fn publish_hulls(owner: Owner, hulls: &[GameHull], adapter: &rustvr_adapter::Adapter) -> bool {
    let mut h = HULLS.lock().unwrap();
    let h = h.get_or_insert_with(|| Hulls { weapons: Vec::new(), items: Vec::new(), table: bytemuck::zeroed_box() });
    let mine = match owner {
        Owner::Weapons => &mut h.weapons,
        Owner::Items => &mut h.items,
    };
    mine.clear();
    mine.extend(hulls.iter().filter(|x| x.id != 0 && x.count >= 4).copied());
    h.table.count = 0;
    for x in h.weapons.iter().chain(&h.items).take(MAX_GAME_HULLS) {
        h.table.hulls[h.table.count as usize] = *x;
        h.table.count += 1;
    }
    adapter.write_game_hulls(&h.table)
}

/// Replaces `owner`'s objects and writes the whole table to the runtime.
pub fn publish(owner: Owner, descs: &[ObjectDesc], adapter: &rustvr_adapter::Adapter) -> bool {
    let mut t = TABLE.lock().unwrap();
    let t = t.get_or_insert_with(|| Table { parts: Parts::default(), table: bytemuck::zeroed_box() });
    t.parts.set(owner, descs, &mut t.table);
    adapter.write_objects(&t.table)
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
