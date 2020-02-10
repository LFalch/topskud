use crate::ext::FloatExt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Health {
    pub hp: f32,
    pub armour: f32,
}

impl Health {
    pub fn weapon_damage(&mut self, dmg: f32, penetration: f32) {
        let frac = (self.armour / 100.).limit(0., 1.);
        debug_assert!(0. < penetration && penetration < 1.);
        let dmg_armour = (1. - penetration) * dmg * frac;
        let dmg_hp = dmg - dmg_armour;

        self.hp -= dmg_hp;
        self.armour -= dmg_armour;
        if self.armour < 0. {
            self.hp += self.armour;
            self.armour = 0.;
        }
    }
    #[inline]
    pub fn is_dead(self) -> bool {
        self.hp <= 0.
    }
}

impl Default for Health {
    fn default() -> Self {
        Self {
            hp: 100.,
            armour: 5.,
        }
    }
}
