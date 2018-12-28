#[derive(Debug, Clone, Copy)]
pub struct Health {
    pub hp: f32,
    pub armour: f32,
}

impl Health {
    pub fn weapon_damage(&mut self, dmg: f32, penetration: f32) {
        let frac = self.armour / 100.;
        let dmg_hp = (1. + (penetration - 1.) * frac) * dmg;
        let dmg_armour = (1. - penetration) * dmg * frac;

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
            armour: 0.,
        }
    }
}
