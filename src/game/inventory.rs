use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemKind {
    Seed(CropSeed),
    Crop(HarvestedCrop),
    Forage(ForageKind),
    Fish(FishKind),
    Ore(OreKind),
    /// A pendant used to propose marriage. Bought at the shop, consumed on proposal.
    Pendant,
    /// Egg laid by a chicken. Can be sold.
    Egg,
    /// Milk from a cow. Can be sold.
    Milk,
    /// Fiber from scything long grass. Can be sold.
    Fiber,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OreKind {
    Copper,
    Iron,
    Gold,
}

impl OreKind {
    pub fn name(&self) -> &'static str {
        match self {
            OreKind::Copper => "copper",
            OreKind::Iron   => "iron",
            OreKind::Gold   => "gold",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FishKind {
    Bass,
    Catfish,
    Trout,
    Salmon,
}

impl FishKind {
    pub fn name(&self) -> &'static str {
        match self {
            FishKind::Bass    => "bass",
            FishKind::Catfish => "catfish",
            FishKind::Trout   => "trout",
            FishKind::Salmon  => "salmon",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ForageKind {
    Mushroom,
    Berry,
    Herb,
    Flower,
    Fern,
    Acorn,
}

impl ForageKind {
    pub fn name(&self) -> &'static str {
        match self {
            ForageKind::Mushroom => "mushroom",
            ForageKind::Berry    => "berry",
            ForageKind::Herb     => "herb",
            ForageKind::Flower   => "flower",
            ForageKind::Fern     => "fern",
            ForageKind::Acorn    => "acorn",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CropSeed {
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

impl CropSeed {
    pub fn name(&self) -> &'static str {
        match self {
            CropSeed::Parsnip     => "parsnip",
            CropSeed::Potato      => "potato",
            CropSeed::Cauliflower => "cauliflower",
            CropSeed::Melon       => "melon",
            CropSeed::Blueberry   => "blueberry",
            CropSeed::Tomato      => "tomato",
            CropSeed::Pumpkin     => "pumpkin",
            CropSeed::Yam         => "yam",
            CropSeed::Cranberry   => "cranberry",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HarvestedCrop {
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

impl HarvestedCrop {
    pub fn name(&self) -> &'static str {
        match self {
            HarvestedCrop::Parsnip     => "parsnip",
            HarvestedCrop::Potato      => "potato",
            HarvestedCrop::Cauliflower => "cauliflower",
            HarvestedCrop::Melon       => "melon",
            HarvestedCrop::Blueberry   => "blueberry",
            HarvestedCrop::Tomato      => "tomato",
            HarvestedCrop::Pumpkin     => "pumpkin",
            HarvestedCrop::Yam         => "yam",
            HarvestedCrop::Cranberry   => "cranberry",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlayerInventory {
    items: HashMap<ItemKind, u32>,
}

impl PlayerInventory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, item: ItemKind, qty: u32) {
        *self.items.entry(item).or_insert(0) += qty;
    }

    pub fn remove(&mut self, item: &ItemKind, qty: u32) -> bool {
        if let Some(count) = self.items.get_mut(item) {
            if *count >= qty {
                *count -= qty;
                if *count == 0 {
                    self.items.remove(item);
                }
                return true;
            }
        }
        false
    }

    pub fn count(&self, item: &ItemKind) -> u32 {
        *self.items.get(item).unwrap_or(&0)
    }

    pub fn items(&self) -> &HashMap<ItemKind, u32> {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_count_item() {
        let mut inv = PlayerInventory::new();
        inv.add(ItemKind::Seed(CropSeed::Parsnip), 5);
        assert_eq!(inv.count(&ItemKind::Seed(CropSeed::Parsnip)), 5);
    }

    #[test]
    fn remove_item_succeeds_when_enough() {
        let mut inv = PlayerInventory::new();
        inv.add(ItemKind::Seed(CropSeed::Parsnip), 5);
        assert!(inv.remove(&ItemKind::Seed(CropSeed::Parsnip), 3));
        assert_eq!(inv.count(&ItemKind::Seed(CropSeed::Parsnip)), 2);
    }

    #[test]
    fn remove_item_fails_when_not_enough() {
        let mut inv = PlayerInventory::new();
        inv.add(ItemKind::Seed(CropSeed::Parsnip), 2);
        assert!(!inv.remove(&ItemKind::Seed(CropSeed::Parsnip), 5));
        assert_eq!(inv.count(&ItemKind::Seed(CropSeed::Parsnip)), 2);
    }
}
