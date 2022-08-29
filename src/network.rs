use serde::{Deserialize, Serialize};
use crate::entity::*;
#[derive(Serialize, Deserialize)]
pub struct SendState {
    pub player: Entity
}
