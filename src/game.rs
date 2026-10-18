//! Read-only consumers of the shared services. No heap addresses are retained.
use bbmod::{
    equipment::EquipmentSnapshot,
    inventory::{InventoryEntry, InventorySnapshot, AVAILABLE, ID_RESOLVED},
    items, Cx, Error,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Melee,
    Gun,
    Sprayer,
    Shield,
    Torch,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Profile {
    pub name: &'static str,
    pub kind: Kind,
    pub rest: u8,
    pub force: u8,
    pub charge_ms: u64,
}
/// Exact SDK catalog codes select authored feel presets, not game weight or attack timing.
pub fn profile(code: &str) -> Option<Profile> {
    let item = items::by_key(code)?;
    if item.category != items::Category::Weapon {
        return None;
    }
    // Strip upgrade labels only after the SDK has confirmed an exact catalog row.
    let base = item
        .key
        .split_once("_plus_")
        .map_or(item.key, |(base, _)| base);
    let (kind, rest, force, charge_ms) = match base {
        "blade_of_mercy" | "lost_blade_of_mercy" | "uncanny_blade_of_mercy" => {
            (Kind::Melee, 12, 85, 550)
        }
        "hunter_axe" | "lost_hunter_axe" | "uncanny_hunter_axe" => (Kind::Melee, 45, 190, 1100),
        "burial_blade" | "lost_burial_blade" | "uncanny_burial_blade" => {
            (Kind::Melee, 30, 150, 950)
        }
        "hunter_blunderbuss" => (Kind::Gun, 0, 125, 900),
        "ludwigs_rifle" => (Kind::Gun, 0, 140, 900),
        "lost_saw_cleaver" | "saw_cleaver" | "uncanny_saw_cleaver" => (Kind::Melee, 25, 140, 750),
        "lost_saw_spear" | "saw_spear" | "uncanny_saw_spear" => (Kind::Melee, 22, 130, 800),
        "kirkhammer" | "lost_kirkhammer" | "uncanny_kirkhammer" => (Kind::Melee, 55, 210, 1200),
        "ludwigs_holy_blade" | "ludwigs_lost_holy_blade" | "ludwigs_uncanny_holy_blade" => {
            (Kind::Melee, 38, 175, 1000)
        }
        "beast_claw" | "lost_beast_claw" | "uncanny_beast_claw" => (Kind::Melee, 14, 100, 600),
        "lost_rifle_spear" | "rifle_spear" | "uncanny_rifle_spear" => (Kind::Melee, 28, 145, 950),
        "lost_reiterpallasch" | "reiterpallasch" | "uncanny_reiterpallasch" => {
            (Kind::Melee, 20, 125, 750)
        }
        "lost_stake_driver" | "stake_driver" | "uncanny_stake_driver" => {
            (Kind::Melee, 40, 210, 1500)
        }
        "logarius_lost_wheel" | "logarius_uncanny_wheel" | "logarius_wheel" => {
            (Kind::Melee, 60, 220, 1300)
        }
        "lost_tonitrus" | "tonitrus" | "uncanny_tonitrus" => (Kind::Melee, 35, 165, 900),
        "hunter_pistol" => (Kind::Gun, 0, 85, 900),
        "evelyn" => (Kind::Gun, 0, 70, 900),
        "repeating_pistol" => (Kind::Gun, 0, 120, 900),
        "cannon" => (Kind::Gun, 0, 200, 900),
        "rosmarinus" => (Kind::Sprayer, 0, 55, 900),
        "flamesprayer" => (Kind::Sprayer, 0, 65, 900),
        "wooden_shield" => (Kind::Shield, 30, 75, 900),
        "hunters_torch" => (Kind::Torch, 8, 20, 900),
        "torch" => (Kind::Torch, 6, 15, 900),
        "lost_threaded_cane" | "threaded_cane" | "uncanny_threaded_cane" => {
            (Kind::Melee, 15, 105, 650)
        }
        _ => return None,
    };
    Some(Profile {
        name: item.name?,
        kind,
        rest,
        force,
        charge_ms,
    })
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Loadout {
    pub generation: u64,
    pub right: Option<&'static str>,
    pub left: Option<&'static str>,
    pub ammo: Option<u32>,
}
impl Loadout {
    pub fn right_profile(&self) -> Option<Profile> {
        self.right.and_then(profile)
    }
    pub fn left_profile(&self) -> Option<Profile> {
        self.left.and_then(profile)
    }
    pub fn summary(&self) -> String {
        let name = |id: Option<&str>| {
            id.map(|id| {
                profile(id)
                    .map(|p| p.name.to_string())
                    .unwrap_or_else(|| format!("Unknown item {id}"))
            })
            .unwrap_or("Unresolved selection".into())
        };
        format!(
            "Strong attack: {} | Left hand: {}\nCarried ammunition: {}",
            name(self.right),
            name(self.left),
            self.ammo
                .map(|n| if n == 0 {
                    "EMPTY — firearm trigger slack".into()
                } else {
                    n.to_string()
                })
                .unwrap_or("unavailable (not treated as zero)".into())
        )
    }
}
fn resolve(reference: u32, entries: &[InventoryEntry]) -> Option<&'static str> {
    if reference == 0 {
        return None;
    }
    let mut matches = entries.iter().filter(|e| e.reference == reference);
    let e = matches.next()?;
    if matches.next().is_some() || e.flags & ID_RESOLVED == 0 {
        return None;
    }
    items::by_id(e.encoded_item_id).map(|item| item.key)
}
/// Use only a complete revision-consistent snapshot. Unresolved entries could
/// contain ammunition: absence is not zero in that case. Unknown catalog IDs
/// also leave ammunition unavailable rather than falsely reporting empty.
pub fn ammunition(entries: &[InventoryEntry]) -> Option<u32> {
    if entries
        .iter()
        .any(|e| e.flags & ID_RESOLVED == 0 || items::by_id(e.encoded_item_id).is_none())
    {
        return None;
    }
    entries
        .iter()
        .filter(|e| {
            matches!(
                e.encoded_item_id,
                items::ids::QUICKSILVER_BULLETS | items::ids::BLOOD_BULLETS
            )
        })
        .try_fold(0u32, |n, e| n.checked_add(e.quantity))
}
pub fn from_snapshots(eq: EquipmentSnapshot, inv: &InventorySnapshot) -> Result<Loadout, Error> {
    if eq.flags & 1 == 0
        || inv.snapshot.flags & AVAILABLE == 0
        || eq.generation != inv.snapshot.generation
        || eq.actor != inv.snapshot.actor
        || eq.character != inv.snapshot.character
        || inv.entries.len() != inv.snapshot.item_count as usize
    {
        return Err(Error::STATE);
    }
    let selected = [eq.selected_even, eq.selected_odd].map(|i| {
        eq.runtime_references
            .get(i as usize)
            .and_then(|r| resolve(*r, &inv.entries))
    });
    // No even=right or odd=left assumption. Only a unique known item category
    // identifies a candidate. Ambiguity/unsupported equipment stays unresolved.
    let choose = |melee: bool| {
        let candidates: Vec<_> = selected
            .iter()
            .flatten()
            .copied()
            .filter(|id| profile(*id).is_some_and(|p| (p.kind == Kind::Melee) == melee))
            .collect();
        if candidates.len() == 1 {
            Some(candidates[0])
        } else {
            None
        }
    };
    Ok(Loadout {
        generation: eq.generation,
        right: choose(true),
        left: choose(false),
        ammo: ammunition(&inv.entries),
    })
}
pub fn read(cx: &Cx) -> Result<Loadout, Error> {
    let before = cx.equipment_snapshot()?;
    let inv = cx.inventory_snapshot()?;
    let after = cx.equipment_snapshot()?;
    if before != after {
        return Err(Error::STATE);
    }
    from_snapshots(after, &inv)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn item(reference: u32, id: u32, quantity: u32) -> InventoryEntry {
        InventoryEntry {
            reference,
            encoded_item_id: id,
            quantity,
            flags: ID_RESOLVED,
            ..Default::default()
        }
    }
    fn snapshots() -> (EquipmentSnapshot, InventorySnapshot) {
        let mut eq = EquipmentSnapshot {
            flags: 1,
            generation: 1,
            actor: 2,
            character: 3,
            selected_even: 0,
            selected_odd: 1,
            ..Default::default()
        };
        eq.runtime_references[0] = 10;
        eq.runtime_references[1] = 20;
        let entries = vec![
            item(10, items::ids::SAW_CLEAVER, 1),
            item(20, items::ids::HUNTER_PISTOL, 1),
            item(30, items::ids::QUICKSILVER_BULLETS, 0),
        ];
        let inv = InventorySnapshot {
            snapshot: bbmod::inventory::InventorySummary {
                flags: 1,
                generation: 1,
                actor: 2,
                character: 3,
                item_count: 3,
                ..Default::default()
            },
            entries,
        };
        (eq, inv)
    }
    #[test]
    fn codes_and_variants() {
        assert_eq!(
            profile("lost_saw_cleaver_plus_10").unwrap().name,
            "Lost Saw Cleaver +10"
        );
        assert_eq!(
            profile("lost_saw_cleaver_plus_10").unwrap().force,
            profile("saw_cleaver").unwrap().force
        );
        assert!(profile("quicksilver_bullets").is_none());
        assert!(profile("saw_cleaver_plus_99").is_none());
        assert!(profile("uncanny_hunter_pistol").is_none());
        assert!(profile("unnamed_weapon_7090000").is_none());
    }
    #[test]
    fn resolve_without_assuming_hand_indices() {
        let (mut eq, inv) = snapshots();
        let a = from_snapshots(eq, &inv).unwrap();
        assert_eq!(a.right, Some("saw_cleaver"));
        assert_eq!(a.left, Some("hunter_pistol"));
        eq.runtime_references.swap(0, 1);
        assert_eq!(from_snapshots(eq, &inv).unwrap(), a);
    }
    #[test]
    fn empty_and_blood_bullets() {
        assert_eq!(ammunition(&[]), Some(0));
        assert_eq!(
            ammunition(&[item(1, items::ids::QUICKSILVER_BULLETS, 0)]),
            Some(0)
        );
        assert_eq!(
            ammunition(&[
                item(1, items::ids::QUICKSILVER_BULLETS, 0),
                item(2, items::ids::BLOOD_BULLETS, 5)
            ]),
            Some(5)
        );
        let mut e = item(1, 0, 0);
        e.flags = 0;
        assert_eq!(ammunition(&[e]), None);
        assert_eq!(ammunition(&[item(1, 0x400003fc, 20)]), None);
    }
    #[test]
    fn stale_and_ambiguous_fail_closed() {
        let (eq, mut inv) = snapshots();
        inv.snapshot.generation += 1;
        assert!(from_snapshots(eq, &inv).is_err());
        inv.snapshot.generation -= 1;
        inv.entries[1].encoded_item_id = items::ids::HUNTER_AXE;
        let l = from_snapshots(eq, &inv).unwrap();
        assert_eq!(l.right, None);
        assert_eq!(l.left, None);
        inv.entries[0].flags = 0;
        assert_eq!(from_snapshots(eq, &inv).unwrap().ammo, None);
    }
}
