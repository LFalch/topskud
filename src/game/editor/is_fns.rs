use super::Insertion;

#[inline]
pub(super) fn is_enemy(ins: Insertion) -> bool {
    if let Insertion::Enemy{..} = ins {
        true
    } else {
        false
    }
}
#[inline]
pub(super) fn is_exit(ins: Insertion) -> bool {
    if let Insertion::Exit = ins {
        true
    } else {
        false
    }
}
#[inline]
pub(super) fn is_intel(ins: Insertion) -> bool {
    if let Insertion::Intel = ins {
        true
    } else {
        false
    }
}
#[inline]
pub(super) fn is_hp(ins: Insertion) -> bool {
    if let Insertion::Pickup(0) = ins {
        true
    } else {
        false
    }
}
#[inline]
pub(super) fn is_armour(ins: Insertion) -> bool {
    if let Insertion::Pickup(1) = ins {
        true
    } else {
        false
    }
}

#[inline]
pub(super) fn is_glock(ins: Insertion) -> bool {
    if let Insertion::Weapon(0) = ins {
        true
    } else {
        false
    }
}
#[inline]
pub(super) fn is_five_seven(ins: Insertion) -> bool {
    if let Insertion::Weapon(1) = ins {
        true
    } else {
        false
    }
}
#[inline]
pub(super) fn is_magnum(ins: Insertion) -> bool {
    if let Insertion::Weapon(2) = ins {
        true
    } else {
        false
    }
}
#[inline]
pub(super) fn is_m4a1(ins: Insertion) -> bool {
    if let Insertion::Weapon(3) = ins {
        true
    } else {
        false
    }
}
#[inline]
pub(super) fn is_ak47(ins: Insertion) -> bool {
    if let Insertion::Weapon(4) = ins {
        true
    } else {
        false
    }
}
#[inline]
pub(super) fn is_arwp(ins: Insertion) -> bool {
    if let Insertion::Weapon(5) = ins {
        true
    } else {
        false
    }
}