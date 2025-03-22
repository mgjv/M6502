use super::*;
use super::cpu::inspect::CpuInspector;

impl Computer {
    pub fn cpu_inspector(&self) -> CpuInspector {
        CpuInspector::new(&self.cpu)
    }
}