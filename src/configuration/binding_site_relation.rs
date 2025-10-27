use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct BindingSiteRelationship {
    pub binding_id: usize,
    pub site_id: usize,
}
