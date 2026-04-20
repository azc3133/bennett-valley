use crate::game::inventory::HarvestedCrop;

#[derive(Debug, Clone, PartialEq)]
pub enum CropKind {
    // Spring
    Parsnip,
    Potato,
    Cauliflower,
    // Summer
    Melon,
    Blueberry,
    Tomato,
    // Fall
    Pumpkin,
    Yam,
    Cranberry,
}

impl CropKind {
    pub fn name(&self) -> &str {
        match self {
            CropKind::Parsnip    => "parsnip",
            CropKind::Potato     => "potato",
            CropKind::Cauliflower => "cauliflower",
            CropKind::Melon      => "melon",
            CropKind::Blueberry  => "blueberry",
            CropKind::Tomato     => "tomato",
            CropKind::Pumpkin    => "pumpkin",
            CropKind::Yam        => "yam",
            CropKind::Cranberry  => "cranberry",
        }
    }

    pub fn to_harvested(&self) -> HarvestedCrop {
        match self {
            CropKind::Parsnip    => HarvestedCrop::Parsnip,
            CropKind::Potato     => HarvestedCrop::Potato,
            CropKind::Cauliflower => HarvestedCrop::Cauliflower,
            CropKind::Melon      => HarvestedCrop::Melon,
            CropKind::Blueberry  => HarvestedCrop::Blueberry,
            CropKind::Tomato     => HarvestedCrop::Tomato,
            CropKind::Pumpkin    => HarvestedCrop::Pumpkin,
            CropKind::Yam        => HarvestedCrop::Yam,
            CropKind::Cranberry  => HarvestedCrop::Cranberry,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CropState {
    pub kind: CropKind,
    pub days_grown: u8,
    pub watered_today: bool,
}

impl CropState {
    pub fn new(kind: CropKind) -> Self {
        Self { kind, days_grown: 0, watered_today: false }
    }

    /// Called at end of day. Returns harvested crop if mature.
    pub fn advance_day(&mut self, grow_days: u8) -> Option<HarvestedCrop> {
        if self.watered_today {
            self.days_grown += 1;
        }
        self.watered_today = false;

        if self.days_grown >= grow_days {
            Some(self.kind.to_harvested())
        } else {
            None
        }
    }

    pub fn is_mature(&self, grow_days: u8) -> bool {
        self.days_grown >= grow_days
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unwatered_crop_does_not_grow() {
        let mut crop = CropState::new(CropKind::Parsnip);
        crop.advance_day(4);
        assert_eq!(crop.days_grown, 0);
    }

    #[test]
    fn watered_crop_grows_one_day() {
        let mut crop = CropState::new(CropKind::Parsnip);
        crop.watered_today = true;
        crop.advance_day(4);
        assert_eq!(crop.days_grown, 1);
    }

    #[test]
    fn watered_flag_resets_after_advance() {
        let mut crop = CropState::new(CropKind::Parsnip);
        crop.watered_today = true;
        crop.advance_day(4);
        assert!(!crop.watered_today);
    }

    #[test]
    fn crop_mature_after_correct_grow_days() {
        let mut crop = CropState::new(CropKind::Parsnip);
        for _ in 0..4 {
            crop.watered_today = true;
            let result = crop.advance_day(4);
            if crop.days_grown < 4 {
                assert!(result.is_none());
            }
        }
        assert!(crop.is_mature(4));
    }

    #[test]
    fn advance_day_returns_harvested_when_mature() {
        let mut crop = CropState::new(CropKind::Parsnip);
        crop.days_grown = 3;
        crop.watered_today = true;
        let result = crop.advance_day(4);
        assert!(result.is_some());
    }
}
