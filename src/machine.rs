use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use crate::config::MachineConfig;

#[derive(Debug, Clone)]
pub struct Machine {
    slots: Vec<Slot>,
    sensor: TemperatureSensor,
}

impl Machine {
    pub fn from_components(config: MachineConfig) -> Machine {
        let mountpoint: PathBuf = config.owfs_mountpoint.into();
        let timing = config.drop_timing;
        let slots = config
            .slot_addresses
            .into_iter()
            .map(|id| Slot::from_components(mountpoint.clone(), id, timing))
            .collect::<Vec<_>>();
        let sensor = TemperatureSensor {
            device: OnewireDevice::from_components(mountpoint.clone(), config.temp_address),
        };

        Machine { slots, sensor }
    }

    pub fn drop(&mut self, slot_idx: usize) -> Result<bool, Box<dyn Error>> {
        if slot_idx >= self.slots.len() {
            return Err(format!("slot index {} out of range", slot_idx).into());
        }

        self.slots[slot_idx].drop()
    }

    pub fn get_active(&self) -> Result<Vec<bool>, Box<dyn Error>> {
        self.slots
            .iter()
            .map(|slot| slot.get_active())
            .collect::<Result<_, _>>()
    }

    pub fn get_bus_ids(&self) -> Vec<String> {
        self.slots
            .iter()
            .map(|slot| slot.device.bus_id.clone())
            .collect()
    }

    pub fn slots(&self) -> usize {
        self.slots.len()
    }

    pub fn get_temperature(&self) -> Result<f32, Box<dyn Error>> {
        self.sensor.get_temperature()
    }
}

#[derive(Debug, Clone)]
struct Slot {
    device: OnewireDevice,
    timing_ms: u64,
}

impl Slot {
    fn from_components(owfs_mountpoint: PathBuf, bus_id: String, timing_ms: u64) -> Slot {
        Slot {
            device: OnewireDevice::from_components(owfs_mountpoint, bus_id),
            timing_ms,
        }
    }

    fn get_active(&self) -> Result<bool, Box<dyn Error>> {
        let id = self.device.read_property("id")?;
        Ok(!id.is_empty())
    }

    fn drop(&mut self) -> Result<bool, Box<dyn Error>> {
        match self.get_active()? {
            true => {
                self.device.write_property("PIO", "1")?;
                thread::sleep(Duration::from_millis(self.timing_ms));
                self.device.write_property("PIO", "0")?;
                Ok(true)
            }
            false => Ok(false),
        }
    }
}

#[derive(Debug, Clone)]
struct TemperatureSensor {
    device: OnewireDevice,
}

impl TemperatureSensor {
    fn get_temperature(&self) -> Result<f32, Box<dyn Error>> {
        let temp_s = self.device.read_property("temperature12")?;
        let temp = temp_s.parse::<f32>()?;
        Ok(temp * (9.0 / 5.0) + 32.0)
    }
}

#[derive(Debug, Clone)]
struct OnewireDevice {
    owfs_mountpoint: PathBuf,
    bus_id: String,
    fs_path: PathBuf,
}

impl OnewireDevice {
    fn from_components(owfs_mountpoint: PathBuf, bus_id: String) -> OnewireDevice {
        let fs_path = owfs_mountpoint.join(&bus_id);

        OnewireDevice {
            owfs_mountpoint,
            bus_id,
            fs_path,
        }
    }

    fn read_property(&self, name: &str) -> Result<String, Box<dyn Error>> {
        let property_path = self.fs_path.join(name);
        let property = fs::read_to_string(&property_path)?;
        Ok(property)
    }

    fn write_property(&mut self, name: &str, value: &str) -> Result<(), Box<dyn Error>> {
        let property_path = self.fs_path.join(name);
        fs::write(&property_path, value)?;
        Ok(())
    }
}
