use crate::{
    util::{Point2, Vector2},
    io::tex::{Assets, },
    io::save::Point2Def,
    obj::{
        Object,
        player::Player,
        enemy::Enemy,
        health::Health,
        bullet::Bullet,
        grenade::Grenade,
        weapon::{WeaponInstance, WeaponDrop, WEAPONS},
        pickup::Pickup,
        decal::{Decal, OldDecoration},
    }
};
use ggez::{
    Context, GameResult,
    error::GameError,
};

use std::collections::btree_map::{BTreeMap, Range, RangeMut};
use std::path::Path;
use std::fs::File;
use std::io::{Write, BufRead, BufReader};

use bincode;

mod material;
pub use material::*;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
pub enum EntityId {
    Exit,
    Intel(u8),
    Enemy(u8),
    Bullet(u16),
    Grenade(u8),
    WeaponDrop(u8),
    Pickup(u8),
}

union UEntity {
    exit: *mut Point2,
    intel: *mut Point2,
    enemy: *mut Enemy,
    bullet: *mut Bullet<'static>,
    grenade: *mut Grenade,
    weapon_drop: *mut WeaponDrop<'static>,
    pickup: *mut Pickup,
}

pub struct EntityMap {
    inner: BTreeMap<EntityId, UEntity>,
    pub next: Next,
}

#[derive(Debug, Default, Clone)]
pub struct Next {
    pub intel: u8,
    pub enemy: u8,
    pub bullet: u16,
    pub grenade: u8,
    pub weapon_drop: u8,
    pub pickup: u8,
}

#[inline]
fn bpointer<T>(t: T) -> *mut T {
    Box::into_raw(Box::new(t))
}

macro_rules! impl_entity_map {
    ($entitymap:ty; $($f_insert:ident, $f_remove:ident, $f_iter:ident, $f_iter_mut:ident, $id:ty, $idvariant:ident, $entity:ident, $enttype:ty,)*) => {
        impl $entitymap {
            $(
            pub fn $f_insert(&mut self, id: Option<$id>, $entity: $enttype) -> Option<Box<$enttype>> {
                let id = if let Some(id) = id {
                    id
                } else {
                    let mut id = self.next.$entity;
                    while self.inner.contains_key(&EntityId::$idvariant(id)) {
                        id += 1;
                    }
                    self.next.$entity = id + 1;
                    id
                };

                let ret = self.inner.insert(EntityId::$idvariant(id), UEntity { $entity: bpointer($entity) });

                unsafe { ret.map(|UEntity{ $entity }| Box::from_raw($entity)) }
            }
            pub fn $f_remove(&mut self, id: $id) -> Option<Box<$enttype>> {
                let ret = self.inner.remove(&EntityId::$idvariant(id));

                unsafe { ret.map(|UEntity{ $entity }| Box::from_raw($entity)) }
            }
            #[inline]
            pub fn $f_iter(&self) -> impl Iterator<Item=($id, &$enttype)> {
                unsafe {
                    self.inner.range(EntityId::$idvariant(0) ..= EntityId::$idvariant(<$id>::max_value()))
                        .map(|(&EntityId::$idvariant(id), &UEntity { $entity: a })| (id, &*a))
                }
            }
            #[inline]
            pub fn $f_iter_mut(&mut self) -> impl Iterator<Item=($id, &mut $enttype)> {
                unsafe {
                    self.inner.range_mut(EntityId::$idvariant(0) ..= EntityId::$idvariant(<$id>::max_value()))
                        .map(|(&EntityId::$idvariant(id), &mut UEntity { $entity: a })| (id, &mut *a))
                }
            }
            )*
        }
    };
}

impl_entity_map!{EntityMap;
    insert_intel, remove_intel, intel_iter, intel_iter_mut, u8, Intel, intel, Point2,
    insert_enemy, remove_enemy, enemy_iter, enemy_iter_mut, u8, Enemy, enemy, Enemy,
    insert_bullet, remove_bullet, bullet_iter, bullet_iter_mut, u16, Bullet, bullet, Bullet<'static>,
    insert_grenade, remove_grenade, grenade_iter, grenade_iter_mut, u8, Grenade, grenade, Grenade,
    insert_weapon_drop, remove_weapon_drop, weapon_drop_iter, weapon_drop_iter_mut, u8, WeaponDrop, weapon_drop, WeaponDrop<'static>,
    insert_pickup, remove_pickup, pickup_iter, pickup_iter_mut, u8, Pickup, pickup, Pickup,
}

impl EntityMap {
    #[inline]
    pub fn new() -> Self {
        EntityMap {
            inner: BTreeMap::new(),
            next: Next::default(),
        }
    }
    pub fn insert_exit(&mut self, exit: Point2) -> Option<Box<Point2>> {
        let ret = self.inner.insert(EntityId::Exit, UEntity { exit: bpointer(exit) });

        unsafe { ret.map(|UEntity{ exit }| Box::from_raw(exit)) }
    }
    pub fn clear(&mut self) {
        std::mem::replace(&mut self.inner, BTreeMap::new());

        for (key, value) in self.inner.into_iter() {
            unsafe { match (key, value) {
                (EntityId::Exit, UEntity { exit }) => { Box::from_raw(exit); } ,
                (EntityId::Intel(_), UEntity { intel }) => { Box::from_raw(intel); } ,
                (EntityId::Enemy(_), UEntity { enemy }) => { Box::from_raw(enemy); } ,
                (EntityId::Bullet(_), UEntity { bullet }) => { Box::from_raw(bullet); } ,
                (EntityId::Grenade(_), UEntity { grenade }) => { Box::from_raw(grenade); } ,
                (EntityId::WeaponDrop(_), UEntity { weapon_drop }) => { Box::from_raw(weapon_drop); } ,
                (EntityId::Pickup(_), UEntity { pickup }) => { Box::from_raw(pickup); } ,
            }}
        }
    }
    pub fn ranger(&mut self) -> EntityRanger {
        EntityRanger {
            inner: self,
            ref_ranges: Vec::new(),
            mut_ranges: Vec::new(),
        }
    }
    pub fn compare_ranges_mut<R: RangeBounds<EntityId>, R2: RangeBounds<EntityId>>(&mut self, range: R, range2: R2) -> CompareRangesMut<'_> {
        let pointer = self as *mut EntityMap;

        if bounds_overlap(range, range2) {
            panic!("Overlapping bounds");
        }

        let first_set = self.inner.range_mut(range);
        let second_set = unsafe { (*pointer) }.inner.range_mut(range2);

        CompareRangesMut {
            first_set_element: first_set.next().map(|(i, e)| EntityRefMut(i, e)).expect("at least element; fix this"),
            pointer,
            first_set,
            second_set,
            next_index: match range2.start_bound() {
                Bound::Included(a) | Bound::Excluded(a) => a.clone(),
                Bound::Unbounded => EntityId::Exit,
            },
            end_index: match range2.end_bound() {
                Bound::Included(a) => Bound::Included(a.clone()),
                Bound::Excluded(a) => Bound::Excluded(a.clone()),
                Bound::Unbounded => Bound::Unbounded,
            }
        }
    }
}

struct EntityRanger<'a> {
    inner: &'a mut EntityMap,
    ref_ranges: Vec<(Bound<EntityId>, Bound<EntityId>)>,
    mut_ranges: Vec<(Bound<EntityId>, Bound<EntityId>)>,
}

fn to_owned(b: Bound<&EntityId>) -> Bound<EntityId> {
    match b {
        Bound::Included(a) => Bound::Included(a.clone()),
        Bound::Excluded(a) => Bound::Excluded(a.clone()),
        Bound::Unbounded => Bound::Unbounded,
    }
}

impl<'a> EntityRanger<'a> {
    fn range<'b, R: RangeBounds<EntityId>>(&'b mut self, range: R) -> Option<impl Iterator<Item = EntityRef<'a>>> {
        if self.mut_ranges.iter().any(|mut_range| bounds_overlap(*mut_range, range)) {
            return None;
        }
        self.ref_ranges.push((to_owned(range.start_bound()), to_owned(range.end_bound())));

        Some(
            unsafe { (*(self.inner as *mut EntityMap)).inner.range(range).map(|(i, e)| EntityRef(i, e)) } 
        )
    }
    fn range_mut<'b, R: RangeBounds<EntityId> + Clone>(&'b mut self, range: R) -> Option<impl Iterator<Item = EntityRefMut<'a>>> {
        let r = range.clone();
        if self.mut_ranges.iter().chain(self.ref_ranges.iter()).any(|ref_range| bounds_overlap(*ref_range, r)) {
            return None;
        }
        self.mut_ranges.push((to_owned(range.start_bound()), to_owned(range.end_bound())));
        
        Some(
            unsafe { (*(self.inner as *mut EntityMap)).inner.range_mut(range).map(|(i, e)| EntityRefMut(i, e)) } 
        )
    }
}

#[inline]
fn bounds_overlap<R: RangeBounds<EntityId>, R2: RangeBounds<EntityId>>(range: R, range2: R2) -> bool {
    blte(range.start_bound(), range2.end_bound()) && blte(range2.start_bound(), range.end_bound())
}

#[test]
#[cfg(test)]
fn test_bounds_overlap() {
    use self::{bounds_overlap, EntityId::*};

    assert!(!bounds_overlap(Exit..Intel(2), Intel(2)..Intel(3)));
    assert!(bounds_overlap(Exit..=Intel(2), Intel(2)..Intel(3)));
    assert!(!bounds_overlap(Intel(2)..Intel(3), Exit..Intel(2)));
    assert!(bounds_overlap(Intel(2)..Intel(3), Exit..=Intel(2)));
}

fn blte(b1: Bound<&EntityId>, b2: Bound<&EntityId>) -> bool {
    match (b1, b2) {
        (_, Bound::Unbounded) => false,
        (Bound::Excluded(a), Bound::Excluded(b)) | (Bound::Included(a), Bound::Included(b)) => a <= b,
        (Bound::Included(a), Bound::Excluded(b)) | (Bound::Excluded(a), Bound::Included(b)) => a < b, 
    }
}

pub struct EntityRefMut<'a>(pub &'a EntityId, &'a mut UEntity);
pub struct EntityRef<'a>(pub &'a EntityId, &'a UEntity);

macro_rules! deref_functions {
    ($($f_name:ident, $idvariant:ident, $entity:ident, $enttype:ty,)*) => {
        impl<'a> EntityRefMut<'a> {
            $( fn $f_name(&mut self) -> &mut $enttype {
                if let &EntityId::$idvariant(_) = self.0 {
                    unsafe { &mut *(self.1).$entity }
                } else {
                    panic!("Invalid entity type: Tried to cast {:?} as {}", self.0, stringify!($idvariant));
                }
            } )*
        }
        impl<'a> EntityRef<'a> {
            $( fn $f_name(&mut self) -> &$enttype {
                if let &EntityId::$idvariant(_) = self.0 {
                    unsafe { &*(self.1).$entity }
                } else {
                    panic!("Invalid entity type: Tried to cast {:?} as {}", self.0, stringify!($idvariant));
                }
            } )*
        }
    };
}

deref_functions!{
    as_intel, Intel, intel, Point2,
    as_enemy, Enemy, enemy, Enemy,
    as_bullet, Bullet, bullet, Bullet<'static>,
    as_grenade, Grenade, grenade, Grenade,
    as_weapon_drop, WeaponDrop, weapon_drop, WeaponDrop<'static>,
    as_pickup, Pickup, pickup, Pickup,
}

use std::ops::{RangeBounds, Bound};

struct CompareRangesMut<'a> {
    pointer: *mut EntityMap,
    first_set: RangeMut<'a, EntityId, UEntity>,
    first_set_element: EntityRefMut<'a>,
    second_set: RangeMut<'a, EntityId, UEntity>,
    next_index: EntityId,
    end_index: Bound<EntityId>,
}

impl<'a> Iterator for CompareRangesMut<'a> {
    type Item = (EntityRefMut<'a>, EntityRefMut<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((s_key, s_elem)) = self.second_set.next() {
            if s_key > &self.next_index {
                self.next_index = s_key.clone();
            }

            Some((self.first_set_element, EntityRefMut(s_key, s_elem)))
        } else {
            unsafe {
                self.second_set = (*self.pointer).inner.range_mut((Bound::Included(self.next_index), self.end_index));
            }
            self.first_set_element = self.first_set.next().map(|(i, e)| EntityRefMut(i, e))?;

            Some((self.first_set_element, self.second_set.next().map(|(i, e)| EntityRefMut(i, e))?))
        }
    }
}

use std::fmt::{self, Debug};

impl Debug for EntityMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = f.debug_map();
        
        unsafe {
            map.entries(self.inner.range(EntityId::Exit.. EntityId::Enemy(0)).map(|(k, UEntity { intel })| (k, &*intel )));
            map.entries(self.inner.range(EntityId::Enemy(0).. EntityId::Bullet(0)).map(|(k, UEntity { enemy })| (k, &*enemy )));
            map.entries(self.inner.range(EntityId::Bullet(0).. EntityId::Grenade(0)).map(|(k, UEntity { bullet })| (k, &*bullet )));
            map.entries(self.inner.range(EntityId::Grenade(0).. EntityId::WeaponDrop(0)).map(|(k, UEntity { grenade })| (k, &*grenade )));
            map.entries(self.inner.range(EntityId::WeaponDrop(0).. EntityId::Pickup(0)).map(|(k, UEntity { weapon_drop })| (k, &*weapon_drop )));
            map.entries(self.inner.range(EntityId::Pickup(0)..).map(|(k, UEntity { pickup })| (k, &*pickup )));
        }

        map.finish()
    }
}

impl Drop for EntityMap {
    fn drop(&mut self) {
        self.clear()
    }
}

#[derive(Debug)]
/// All the objects in the current world
pub struct World {
    pub player: Player,
    pub palette: Palette,
    pub grid: Grid,
    pub decals: Vec<Decal>,

    pub entities: EntityMap,
}

impl World {
    pub fn enemy_pickup(&mut self) {
        let dead_weapons = Vec::new();
        let dead_pickups = Vec::new();

        let mut ranger = self.entities.ranger();
        let mut weapons = ranger.range(EntityId::WeaponDrop(0)..=EntityId::WeaponDrop(255)).unwrap();
        let mut pickups = ranger.range(EntityId::Pickup(0)..=EntityId::Pickup(255)).unwrap();

        for enemy in ranger.range_mut(EntityId::Enemy(0)..=EntityId::Enemy(255)).unwrap() {
            let enemy = enemy.as_enemy();

            let mut dead = None;
            for weapon in weapons {
                let w = weapon.0;
                let weapon = weapon.as_weapon_drop();
                if (weapon.pos - enemy.pl.obj.pos).norm() <= 16. {
                    dead = Some(w);
                    break;
                }
            }
            if let Some(i) = dead {
                enemy.pl.wep = Some(WeaponInstance::from_drop(self.entities.remove_weapon_drop(i)));
            }
        }


        for (enemy, pickup) in self.entities.compare_ranges_mut(EntityId::Enemy(0)..=EntityId::Enemy(255), EntityId::WeaponDrop(0)..=EntityId::Pickup(255)) {
            let enemy = enemy.as_enemy();
            match pickup.0 {
                EntityId::WeaponDrop(w) => {
                    let weapon_drop = pickup.as_weapon_drop();
                    if (weapon_drop.pos - enemy.pl.obj.pos).norm() <= 16. {

                        if dead_weapons.contains(w) {
                            enemy.pl.wep = Some(WeaponInstance::from_drop(weapon_drop.clone()));
    
                            dead_weapons.push(w);
                        }
                    }

                }
                EntityId::Pickup(p) => {
                    let pickup = pickup.as_pickup();
                    if (pickup.pos - enemy.pl.obj.pos).norm() <= 16. {


                        dead_pickups.push(p);
                    }

                }
                _ => unreachable!(),
            }

            let mut dead = None;
            for (w, weapon) in self.weapons.iter().enumerate() {
                if (weapon.pos - enemy.pl.obj.pos).norm() <= 16. {
                    dead = Some(w);
                    break;
                }
            }
            if let Some(i) = dead {
                enemy.pl.wep = Some(WeaponInstance::from_drop(self.weapons.remove(i)));
            }
            let mut deads = Vec::new();
            for (p, pickup) in self.pickups.iter().enumerate() {
                if (pickup.pos - enemy.pl.obj.pos).norm() <= 16. {
                    deads.push(p);
                    break;
                }
            }
            for i in deads.into_iter() {
                let pickup = self.pickups.remove(i);
                let _action_done = pickup.apply(&mut enemy.pl.health);
            }
        }
    }
    pub fn player_pickup(&mut self) {
        let player = &mut self.player;
        if player.wep.is_none() {
            let mut dead = None;
            for (w, weapon) in self.weapons.iter().enumerate() {
                if (weapon.pos - player.obj.pos).norm() <= 16. {
                    dead = Some(w);
                    break;
                }
            }
            if let Some(i) = dead {
                player.wep = Some(WeaponInstance::from_drop(self.weapons.remove(i)));
            }
        }

        let mut deads = Vec::new();
        for (p, pickup) in self.pickups.iter().enumerate() {
            if (pickup.pos - player.obj.pos).norm() <= 16. {
                deads.push(p);
                break;
            }
        }
        for i in deads.into_iter() {
            let pickup = self.pickups.remove(i);
            let _action_done = pickup.apply(&mut player.health);
        }
    }
}

pub struct Statistics {
    pub time: usize,
    pub enemies_left: usize,
    pub health_left: Health,
    pub level: Level,
    pub weapon: Option<WeaponInstance<'static>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Entity {
    SimpleEnemy {
        obj: Object,
        weapon: usize,
    },
    Enemy {
        obj: Object,
        health: Health,
        weapon: usize,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLevel {
    pub grid: Grid,
    #[serde(with = "Point2Def")]
    pub start_point: Point2,
    #[serde(with = "opt_point")]
    pub exit: Option<Point2>,
    pub entities: Vec<Entity>,
}

mod opt_point {
    use serde::{Serialize, Deserialize, Serializer, Deserializer};
    use crate::util::Point2;

    #[inline]
    pub fn serialize<S: Serializer>(p: &Option<Point2>, s: S) -> Result<S::Ok, S::Error> {
        p.map(|p| (p.x, p.y)).serialize(s)
    }
    #[inline]
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Point2>, D::Error> {
        <Option<(f32, f32)>>::deserialize(d).map(|p| p.map(|(x, y)| Point2::new(x, y)))
    }
}


#[derive(Debug, Clone)]
pub struct Level {
    pub palette: Palette,
    pub grid: Grid,
    pub start_point: Option<Point2>,
    pub enemies: Vec<Enemy>,
    pub exit: Option<Point2>,
    pub intels: Vec<Point2>,
    pub pickups: Vec<(Point2, u8)>,
    pub decals: Vec<Decal>,
    pub weapons: Vec<WeaponDrop<'static>>,
}

impl Level {
    pub fn new(palette: Palette, width: u16, height: u16) -> Self {
        Self {
            palette,
            grid: Grid::new(width, height),
            start_point: None,
            enemies: Vec::new(),
            exit: None,
            intels: Vec::new(),
            pickups: Vec::new(),
            decals: Vec::new(),
            weapons: Vec::new(),
        }
    }
    pub fn load<P: AsRef<Path>>(path: P) -> GameResult<Self> {
        let mut reader = BufReader::new(File::open(path)?);
        let mut ret = Level::new(Palette::default(), 0, 0);

        // For support of older level files
        const WEAPONS_OLD: [&str; 6] = [
            "glock",
            "five_seven",
            "magnum",
            "m4a1",
            "ak47",
            "arwp",
        ];

        loop {
            let mut buf = String::with_capacity(16);
            reader.read_line(&mut buf)?;
            match &*buf.trim_end() {
                "" => continue,
                "PALETTE" => ret.palette = bincode::deserialize_from(&mut reader)
                    .map(|mats: Vec<Box<str>>| Palette::new(mats.into_iter().map(|s| &*Box::leak(s)).collect()))
                    .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?,
                "GRD" => ret.grid = bincode::deserialize_from(&mut reader)
                    .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?,
                "GRID" => {
                    let (w, grid): (usize, Vec<u16>) = bincode::deserialize_from(&mut reader)
                    .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?;
                    ret.grid = Grid {
                        mats: grid.into_iter().map(|n| n as u8).collect(),
                        width: w as u16
                    }
                }
                "START" => ret.start_point = Some(
                    bincode::deserialize_from(&mut reader)
                        .map(|(x, y)| Point2::new(x, y))
                        .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?
                ),
                "ENEMIES" => ret.enemies = bincode::deserialize_from(&mut reader)
                    .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?,
                "POINT GOAL" => ret.exit = Some(bincode::deserialize_from(&mut reader)
                    .map(|(x, y)| Point2::new(x, y))
                    .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?),
                "INTELS" => ret.intels = bincode::deserialize_from(&mut reader)
                    .map(|l: Vec<(f32, f32)>| l.into_iter().map(|(x, y)| Point2::new(x, y)).collect())
                    .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?,
                "DECORATIONS" => ret.decals = bincode::deserialize_from(&mut reader)
                    .map(|old_decs: Vec<OldDecoration>| old_decs.into_iter().map(|od| od.renew()).collect())
                    .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?,
                "DECS" => ret.decals = bincode::deserialize_from(&mut reader)
                    .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?,
                "PICKUPS" => ret.pickups = bincode::deserialize_from(&mut reader)
                    .map(|l: Vec<((f32, f32), u8)>| l.into_iter().map(|((x, y), i)| (Point2::new(x, y), i)).collect())
                    .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?,
                "WEPS" => ret.weapons = bincode::deserialize_from(&mut reader)
                    .map(|l: Vec<((f32, f32), String)>| l.into_iter().map(|((x, y), id)| WEAPONS[&id].make_drop(Point2::new(x, y))).collect())
                    .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?,
                "WEAPONS" => ret.weapons = bincode::deserialize_from(&mut reader)
                    .map(|l: Vec<((f32, f32), u8)>| l.into_iter().map(|((x, y), i)| WEAPONS[WEAPONS_OLD[i as usize]].make_drop(Point2::new(x, y))).collect())
                    .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?,
                "END" => break, 
                _ => return Err(GameError::ResourceLoadError("Bad section".to_string()))
            }
        }

        Ok(ret)
    }
    pub fn save<P: AsRef<Path>>(&self, path: P) -> GameResult<()> {
        let mut file = File::create(path)?;

        writeln!(file, "GRD")?;
        bincode::serialize_into(&mut file, &self.grid)
            .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?;
        if let Some(start) = self.start_point {
            writeln!(file, "\nSTART")?;
            bincode::serialize_into(&mut file, &(start.x, start.y))
            .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?;
        }
        if !self.enemies.is_empty() {
            writeln!(file, "\nENEMIES")?;
            bincode::serialize_into(&mut file, &self.enemies)
            .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?;
        }
        if let Some(p) = self.exit {
            writeln!(file, "\nPOINT GOAL")?;
            bincode::serialize_into(&mut file, &(p.x, p.y))
            .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?;
        }
        if !self.intels.is_empty() {
            writeln!(file, "\nINTELS")?;
            let intels: Vec<_> = self.intels.iter().map(|p| (p.x, p.y)).collect();
            bincode::serialize_into(&mut file, &intels)
                .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?;
        }
        if !self.decals.is_empty() {
            writeln!(file, "\nDECS")?;
            bincode::serialize_into(&mut file, &self.decals)
            .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?;
        }
        if !self.pickups.is_empty() {
            writeln!(file, "\nPICKUPS")?;
            let pickups: Vec<_> = self.pickups.iter().map(|&(p, i)| ((p.x, p.y), i)).collect();
            bincode::serialize_into(&mut file, &pickups)
                .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?;
        }
        if !self.weapons.is_empty() {
            writeln!(file, "\nWEPS")?;
            let pickups: Vec<((f32, f32), String)> = self.weapons.iter().map(|w| ((w.pos.x, w.pos.y), {
                WEAPONS.iter().find(|(_, wep)| wep.name == w.weapon.name).map(|(id, _)| id.clone()).unwrap()
            })).collect();
            bincode::serialize_into(&mut file, &pickups)
                .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?;
        }

        writeln!(file, "\nEND")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid{
    width: u16,
    mats: Vec<u8>,
}

impl Grid {
    pub fn new(width: u16, height: u16) -> Self {
        Grid {
            width,
            mats: vec![0; (width*height) as usize],
        }
    }
    pub fn migrate(&mut self, from: &Palette, to: Palette) -> Palette {
        let to = to.and(from);

        for mat in &mut self.mats {
            *mat = to.find(from.get(*mat).unwrap()).unwrap();
        }

        to
    }
    #[inline]
    pub fn width(&self) -> u16 {
        self.width
    }
    pub fn height(&self) -> u16 {
        self.mats.len() as u16 / self.width
    }
    pub fn widen(&mut self) {
        let width = self.width as usize;
        let height = self.height() as usize;
        self.mats.reserve_exact(height);
        for i in (1..=height).rev().map(|i| i * width) {
            self.mats.insert(i, 0);
        }
        self.width += 1;
    }
    pub fn thin(&mut self) {
        if self.width <= 1 {
            return
        }
        let width = self.width;
        for i in (1..=self.height()).rev().map(|i| i * width - 1) {
            self.mats.remove(i as usize);
        }
        self.width -= 1;
    }
    pub fn heighten(&mut self) {
        let new_len = self.mats.len() + self.width as usize;
        self.mats.reserve_exact(self.width as usize);
        self.mats.resize(new_len, 0);
    }
    pub fn shorten(&mut self) {
        let new_len = self.mats.len() - self.width as usize;
        if new_len == 0 {
            return
        }
        self.mats.truncate(new_len);
    }
    #[inline]
    pub fn snap(c: Point2) -> (u16, u16) {
        Self::snap_coords(c.x, c.y)
    }
    #[inline]
    fn idx(&self, x: u16, y: u16) -> usize {
        x.saturating_add(y.saturating_mul(self.width)) as usize
    }
    pub fn snap_coords(x: f32, y: f32) -> (u16, u16) {
        fn db32omin(n: f32) -> u16 {
            if n < 0. {
                std::u16::MAX
            } else {
                (n / 32.) as u16
            }
        }

        (db32omin(x), db32omin(y))
    }
    pub fn get(&self, x: u16, y: u16) -> Option<u8> {
        if x < self.width {
            self.mats.get(self.idx(x, y)).cloned()
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn is_solid_tuple(&self, pal: &Palette, (x, y): (u16, u16)) -> bool {
        self.is_solid(pal, x, y)
    }
    pub fn is_solid(&self, pal: &Palette, x: u16, y: u16) -> bool {
        self.get(x, y).map(|m| pal.is_solid(m)).unwrap_or(true)
    }
    pub fn insert(&mut self, x: u16, y: u16, mat: u8) {
        if x < self.width {
            let i = self.idx(x, y);
            if let Some(m) = self.mats.get_mut(i) {
                *m = mat;
            }
        }
    }
    pub fn ray_cast(&self, pal: &Palette, from: Point2, dist: Vector2, finite: bool) -> RayCast {
        let dest = from + dist;

        let mut cur = from;
        let mut to_wall = Vector2::new(0., 0.);
        let (mut gx, mut gy) = Self::snap(cur);
        let x_dir = Direction::new(dist.x);
        let y_dir = Direction::new(dist.y);

        loop {
            if finite && (cur - dest).dot(&dist) / dist.norm() >= 0. {
                break RayCast::n_full(dest);
            }

            let mat = self.get(gx, gy);

            if let Some(mat) = mat {
                if pal.is_solid(mat) {
                    break RayCast::n_half(cur, dest-cur, to_wall);
                }
                if cur.x < 0. || cur.y < 0. {
                    break RayCast::n_off_edge(cur, dest-cur); 
                }
            } else {
                break RayCast::n_off_edge(cur, dest-cur);
            }

            let nearest_corner = Point2::new(x_dir.on(f32::from(gx) * 32.), y_dir.on(f32::from(gy) * 32.));
            let distance = nearest_corner - cur;

            let time = (distance.x/dist.x, distance.y/dist.y);

            if time.0 < time.1 {
                to_wall.x = dist.x.signum();
                to_wall.y = 0.;
                // Going along x
                cur.x = nearest_corner.x;
                cur.y += time.0 * dist.y;

                gx = if let Some(n) = x_dir.on_u16(gx) {
                    n
                } else {
                    break RayCast::n_off_edge(cur, dest-cur);
                }
            } else {
                if time.0 - time.1 < std::f32::EPSILON {
                    to_wall.x = dist.x.signum();
                    to_wall.y = dist.y.signum();
                } else {
                    to_wall.x = 0.;
                    to_wall.y = dist.y.signum();
                }
                // Going along y
                cur.y = nearest_corner.y;
                cur.x += time.1 * dist.x;

                gy = if let Some(n) = y_dir.on_u16(gy) {
                    n
                } else {
                    break RayCast::n_off_edge(cur, dest-cur);
                }
            }
        }
    }
    /// Closest point on a line segment to a circle
    pub fn closest_point_of_line_to_circle(line_start: Point2, line_dist: Vector2, circle_center: Point2) -> Point2 {
        let c = circle_center - line_start;

        let d_len = line_dist.norm();

        let c_on_d_len = c.dot(&line_dist) / d_len;

        if c_on_d_len < 0. {
            // Closest point is start point
            line_start
        } else if c_on_d_len <= d_len {
            // Closest point is betweeen start and end point
            let c_on_d = c_on_d_len / d_len * line_dist;
            line_start + c_on_d
        } else {
            // Closest point is end point
            line_start + line_dist
        }
    }
    /// Distance between a line section and a circle
    /// 
    /// The general formula for distance between a line and cirlcle here would be inadequate
    /// since here the line has a finite length so we need to check if the smalleset distance is in that finite line section.
    #[inline]
    pub fn distance_line_circle(line_start: Point2, line_dist: Vector2, circle_center: Point2) -> Vector2 {
        let closest_point = Self::closest_point_of_line_to_circle(line_start, line_dist, circle_center);

        circle_center - closest_point
    }
    /// Length of `distance_line_circle`
    #[inline]
    pub fn dist_line_circle(line_start: Point2, line_dist: Vector2, circle_center: Point2) -> f32 {
        Self::distance_line_circle(line_start, line_dist, circle_center).norm()
    }
    pub fn draw(&self, pal: &Palette, ctx: &mut Context, assets: &Assets) -> GameResult<()> {
        for (i, &mat) in self.mats.iter().enumerate() {
            let x = f32::from(i as u16 % self.width) * 32.;
            let y = f32::from(i as u16 / self.width) * 32.;

            pal.draw_mat(mat, ctx, assets, x, y, Default::default())?;
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
enum Direction {
    Pos,
    Neg,
}

impl Direction {
    #[inline]
    fn new(n: f32) -> Self {
        if n.is_sign_negative() {
            Direction::Neg
        } else {
            Direction::Pos
        }
    }
    #[inline]
    fn on_u16(self, n: u16) -> Option<u16> {
        match self {
            Direction::Pos => Some(n + 1),
            Direction::Neg => n.checked_sub(1),
        }
    }
    #[inline]
    fn on(self, n: f32) -> f32 {
        match self {
            Direction::Pos => n + 32.,
            Direction::Neg => n,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RayCast {
    result: RayCastResult,
    point: Point2,
    clip: Vector2,
}

#[derive(Debug, Copy, Clone)]
enum RayCastResult {
    Full,
    Half(Vector2),
    OffEdge,
}

impl RayCast {
    fn n_full(point: Point2) -> Self {
        RayCast{
            result: RayCastResult::Full,
            point,
            clip: Vector2::new(0., 0.)
        }
    }
    fn n_half(point: Point2, clip: Vector2, to_wall: Vector2) -> Self {
        RayCast{
            result: RayCastResult::Half(to_wall),
            point,
            clip,
        }
    }
    fn n_off_edge(point: Point2, clip: Vector2) -> Self {
        RayCast{
            result: RayCastResult::OffEdge,
            point,
            clip,
        }
    }

    pub fn full(self) -> bool {
        match self.result {
            RayCastResult::Full => true,
            _ => false,
        }
    }
    pub fn half(self) -> bool {
        match self.result {
            RayCastResult::Half(_) => true,
            _ => false,
        }
    }
    pub fn half_vec(self) -> Option<Vector2> {
        match self.result {
            RayCastResult::Half(v) => Some(v),
            _ => None,
        }
    }
    pub fn into_point(self) -> Point2 {
        let Self{point, ..} = self;
        point
    }
    pub fn clip(self) -> Vector2 {
        let Self{clip, ..} = self;
        clip
    }
}