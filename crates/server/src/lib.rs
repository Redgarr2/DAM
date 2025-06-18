//! LAN server for asset sharing

use schema::DamResult;

pub struct ServerService;

impl ServerService {
    pub fn new() -> DamResult<Self> {
        Ok(Self)
    }
}
