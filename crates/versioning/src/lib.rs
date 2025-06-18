//! Version control for digital assets

use schema::DamResult;

pub struct VersioningService;

impl VersioningService {
    pub fn new() -> DamResult<Self> {
        Ok(Self)
    }
}
