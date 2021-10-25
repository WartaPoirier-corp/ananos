use acpi::PciConfigRegions;
use alloc::collections::BTreeMap;
use pci_types::{Bar, ConfigRegionAccess, EndpointHeader, PciAddress, PciHeader};
use x86_64::VirtAddr;
use crate::memory::MEM_OFFSET;

/// S/O https://github.com/IsaacWoods/pebble/blob/main/kernel/kernel_x86_64/src/pci.rs

pub struct ConfigAccess(pub PciConfigRegions);

impl ConfigRegionAccess for ConfigAccess {
    fn function_exists(&self, address: pci_types::PciAddress) -> bool {
        self.0.physical_address(address.segment(), address.bus(), address.device(), address.device()).is_some()
    }

    unsafe fn read(&self, address: pci_types::PciAddress, offset: u16) -> u32 {
        let phys = self.0.physical_address(address.segment(), address.bus(), address.device(), address.function()).unwrap();        
        let ptr = VirtAddr::new(MEM_OFFSET + phys + offset as u64).as_ptr();
        core::ptr::read_volatile(ptr)
    }

    unsafe fn write(&self, address: pci_types::PciAddress, offset: u16, value: u32) {
        let phys = self.0.physical_address(address.segment(), address.bus(), address.device(), address.function()).unwrap();        
        let ptr = VirtAddr::new(MEM_OFFSET + phys + offset as u64).as_mut_ptr();
        core::ptr::write_volatile(ptr, value);
    }
}

pub struct PciDevice {
    pub vendor_id: u16,
    pub device_id: u16,
    pub revision: u8,
    pub class: u8,
    pub sub_class: u8,
    pub interface: u8,
    pub bars: [Option<Bar>; 6],
}

pub struct PciInfo {
    devices: BTreeMap<PciAddress, PciDevice>,
}

pub struct PciResolver<A> where A: ConfigRegionAccess {
    access: A,
    info: PciInfo,
}

impl<A> PciResolver<A> where A: ConfigRegionAccess {
    pub fn get_info(access: A) -> PciInfo {
        let mut resolver = PciResolver {
            access,
            info: PciInfo { devices: BTreeMap::new() }
        };

        if PciHeader::new(PciAddress::new(0, 0, 0, 0)).has_multiple_functions(&resolver.access) {
            for i in 0..8 {
                resolver.check_bus(i);
            }
        } else {
            resolver.check_bus(0);
        }

        resolver.info
    }

    fn check_bus(&mut self, bus: u8) {
        for device in 0..32 {
            let address = PciAddress::new(0, bus, device, 0);
            if self.access.function_exists(address) {
                self.check_function(bus, device, 0);
                let header = PciHeader::new(address);
                if header.has_multiple_functions(&self.access) {
                    // The device is multi-function. We need to check the rest.
                    for function in 1..8 {
                        self.check_function(bus, device, function);
                    }
                }
            }
        }
    }

    fn check_function(&mut self, bus: u8, device: u8, function: u8) {
        let address = PciAddress::new(0, bus, device, function);
        if self.access.function_exists(address) {
            let header = PciHeader::new(address);
            let (vendor_id, device_id) = header.id(&self.access);
            let (revision, class, sub_class, interface) = header.revision_and_class(&self.access);

            if vendor_id == 0xffff {
                return;
            }

            crate::println!(
                "Found PCI device (bus={}, device={}, function={}): (vendor = {:#x}, device = {:#x})",
                bus, device, function, vendor_id, device_id
            );

            match header.header_type(&self.access) {
                pci_types::HEADER_TYPE_ENDPOINT => {
                    let endpoint_header = EndpointHeader::from_header(header, &self.access).unwrap();
                    let bars = {
                        let mut bars = [None; 6];

                        let mut skip_next = false;
                        for i in 0..6 {
                            if skip_next {
                                continue;
                            }

                            let bar = endpoint_header.bar(i, &self.access);
                            skip_next = match bar {
                                Some(Bar::Memory64 { .. }) => true,
                                _ => false,
                            };
                            bars[i as usize] = bar;
                        }

                        bars
                    };

                    self.info.devices.insert(
                        address,
                        PciDevice { vendor_id, device_id, revision, class, sub_class, interface, bars },
                    );
                }
                reserved => panic!("PCI function has reserved header type: {:#x}", reserved),
            }
        }
    }
}

