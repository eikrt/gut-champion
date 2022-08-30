use serde::{Deserialize, Serialize};
use crate::entity::*;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SendState {
    pub id: u64,
    pub player: Entity,
}
