use crate::strings::FixedSizeString;

pub type MarketID = FixedSizeString<11>;
pub type EventID = u32;
pub type EventTypeID = u32;

pub type Clk = staticvec::StaticString<24>; // can be variable length

#[derive(Default, Clone, Copy, PartialEq)]
pub struct SelectionID(u32, Option<f32>);

impl From<(u32, Option<f32>)> for SelectionID {
    fn from(v: (u32, Option<f32>)) -> Self {
        Self(v.0, v.1)
    }
}

impl SelectionID {
    pub fn id(&self) -> u32 {
        self.0
    }

    pub fn handicap(&self) -> Option<f32> {
        self.1
    }
}
