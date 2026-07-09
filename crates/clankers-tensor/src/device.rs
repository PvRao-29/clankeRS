//! Device placement for tensors.

/// Where a tensor's memory lives.
///
/// Only [`Device::Cpu`] is functional in this release. The accelerator variants
/// exist so the public API (specs, engine builder, error messages) is already
/// shaped for them; constructing a tensor on a non-CPU device is accepted at the
/// type level but backends will reject it until a corresponding backend ships.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Device {
    /// Host memory. The only device with real support today.
    #[default]
    Cpu,
    /// NVIDIA GPU with the given ordinal. Stubbed — no backend consumes it yet.
    Cuda(u32),
    /// Apple Metal GPU. Stubbed.
    Metal,
}

impl Device {
    /// Whether this device is the host CPU.
    pub const fn is_cpu(self) -> bool {
        matches!(self, Device::Cpu)
    }

    /// Short label for diagnostics.
    pub fn label(self) -> String {
        match self {
            Device::Cpu => "cpu".to_string(),
            Device::Cuda(i) => format!("cuda:{i}"),
            Device::Metal => "metal".to_string(),
        }
    }

    /// Parse a device from a config string (case-insensitive).
    ///
    /// Accepts `cpu`, `metal`/`mps`, and `cuda` / `cuda:N` / `cuda0`.
    pub fn parse(s: &str) -> Option<Device> {
        let s = s.trim().to_ascii_lowercase();
        match s.as_str() {
            "cpu" => Some(Device::Cpu),
            "metal" | "mps" => Some(Device::Metal),
            other => {
                let rest = other.strip_prefix("cuda")?;
                let idx = rest.trim_start_matches([':', '_', ' ']);
                if idx.is_empty() {
                    Some(Device::Cuda(0))
                } else {
                    idx.parse::<u32>().ok().map(Device::Cuda)
                }
            }
        }
    }
}

impl std::fmt::Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_spellings() {
        assert_eq!(Device::parse("cpu"), Some(Device::Cpu));
        assert_eq!(Device::parse("CPU"), Some(Device::Cpu));
        assert_eq!(Device::parse("metal"), Some(Device::Metal));
        assert_eq!(Device::parse("mps"), Some(Device::Metal));
        assert_eq!(Device::parse("cuda"), Some(Device::Cuda(0)));
        assert_eq!(Device::parse("cuda:1"), Some(Device::Cuda(1)));
        assert_eq!(Device::parse("cuda0"), Some(Device::Cuda(0)));
        assert_eq!(Device::parse("tpu"), None);
    }
}
