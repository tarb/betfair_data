use crate::strings::FixedSizeString;

pub type MarketID = FixedSizeString<11>;
pub type EventID = u32;
pub type SelectionID = u32;
pub type EventTypeID = u32;

pub type Clk = staticvec::StaticString<24>; // can be variable length 
