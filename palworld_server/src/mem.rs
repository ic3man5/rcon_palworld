use psutil::memory::{os::linux::VirtualMemoryExt, virtual_memory};
use anyhow::Result;
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MemInfo {
    pub mem_total: u64,
    pub mem_free: u64,
    pub mem_available: u64,
    pub buffers: u64,
    pub cached: u64,
}

impl MemInfo {
    pub fn used(&self) -> Option<u64> {
        self.mem_total.checked_sub(self.mem_available)
    }

    pub fn used_percent(&self) -> Option<f64> {
        match self.used() {
            Some(used) => {
                let used = used as f64;
                Some(used / (self.mem_total as f64))
            }
            None => None,
        }
    }

    pub fn get_memory_info() -> Result<Self> {
        let virt_mem = virtual_memory()?;
        Ok(Self {
            mem_total: virt_mem.total(),
            mem_free: virt_mem.free(),
            mem_available: virt_mem.available(),
            buffers: virt_mem.buffers(),
            cached: virt_mem.cached(),
        })
    }
}
