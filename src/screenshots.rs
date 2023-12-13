use super::*;

pub struct Screenshots<Manager> {
    pub(crate) screenshots: *mut sys::ISteamScreenshots,
    pub(crate) inner: Arc<Inner<Manager>>,
}
