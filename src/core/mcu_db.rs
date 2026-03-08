use crate::mcu;

#[derive(Debug, Clone)]
pub struct McuInfo {
    pub id: &'static str,
    pub part_number: &'static str,
    pub define: &'static str,
    pub core: &'static str,
    pub fpu: &'static str,
    pub clock_mhz: u32,
    pub flash_kb: u32,
    pub ram_kb: u32,
    #[allow(dead_code)]
    pub family: &'static str,
    #[allow(dead_code)]
    pub series: &'static str,
    pub vendor: &'static str,
}

impl McuInfo {
    pub fn flash_str(&self) -> String {
        if self.flash_kb >= 1024 {
            format!("{}MB", self.flash_kb / 1024)
        } else {
            format!("{}KB", self.flash_kb)
        }
    }

    pub fn ram_str(&self) -> String {
        format!("{}KB", self.ram_kb)
    }
}

/// Lookup MCU by alias (e.g., "k64")
pub fn lookup(id: &str) -> Option<&'static McuInfo> {
    let id_lower = id.to_lowercase();
    all_mcus().into_iter().find(|m| m.id == id_lower)
}

/// All supported MCUs
pub fn list_all() -> Vec<&'static McuInfo> {
    all_mcus()
}

/// Filter by family
#[allow(dead_code)]
pub fn list_by_family(family: &str) -> Vec<&'static McuInfo> {
    all_mcus()
        .into_iter()
        .filter(|m| m.family.eq_ignore_ascii_case(family))
        .collect()
}

/// Supported MCU alias list (for error messages)
pub fn supported_ids() -> Vec<&'static str> {
    all_mcus().into_iter().map(|m| m.id).collect()
}

fn all_mcus() -> Vec<&'static McuInfo> {
    let mut all: Vec<&'static McuInfo> = Vec::new();
    all.extend(mcu::nxp::NXP_MCUS.iter());
    // TODO: add STM32 MCUs here
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_existing() {
        let mcu = lookup("k64").unwrap();
        assert_eq!(mcu.define, "MK64F12");
        assert_eq!(mcu.core, "cortex-m4");
        assert_eq!(mcu.flash_kb, 1024);
    }

    #[test]
    fn test_lookup_case_insensitive() {
        assert!(lookup("K64").is_some());
        assert!(lookup("K22F").is_some());
    }

    #[test]
    fn test_lookup_missing() {
        assert!(lookup("stm32f4").is_none());
    }

    #[test]
    fn test_list_all() {
        let all = list_all();
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_list_by_family() {
        let kinetis = list_by_family("kinetis");
        assert_eq!(kinetis.len(), 5);
    }

    #[test]
    fn test_supported_ids() {
        let ids = supported_ids();
        assert!(ids.contains(&"k64"));
        assert!(ids.contains(&"k66"));
    }

    #[test]
    fn test_flash_str() {
        let mcu = lookup("k64").unwrap();
        assert_eq!(mcu.flash_str(), "1MB");

        let mcu = lookup("k10d").unwrap();
        assert_eq!(mcu.flash_str(), "512KB");
    }
}
