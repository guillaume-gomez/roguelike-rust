use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
/// An object that can be equipped, yielding bonuses.
pub struct Equipment {
  pub slot: Slot,
  pub equipped: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Slot {
  LeftHand,
  RightHand,
  Head,
}

impl fmt::Display for Slot {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      Slot::LeftHand => write!(f, "Left hand"),
      Slot::RightHand => write!(f, "Right hand"),
      Slot::Head => write!(f, "Head")
    }
  }
}

impl fmt::Display for Equipment {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "({}, {})", self.slot, self.equipped)
  }
}
