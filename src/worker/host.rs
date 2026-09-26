//! The host as the receipt records it (`HostV1`, B14a-2c R16 round 2 decision 3): each fact read
//! from one bounded file and parsed by one pure function, so the reading and the rule are separate
//! (F95) and the raw bytes are kept as the receipt's `facts` payload.

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// The most bytes one host fact file may hold; `/proc/cpuinfo` on a 16-way host measures ~24 KiB.
pub const MAX_FACT_BYTES: usize = 1024 * 1024;

/// Why a host fact could not be read or parsed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// A fact file could not be opened or read.
    Io,
    /// A fact file is larger than [`MAX_FACT_BYTES`].
    Bound,
    /// A fact file does not carry the field the parser names.
    Missing,
}

/// One host's facts, as read: the operating system's id and version, the binary's architecture,
/// the kernel line, the boot id, the logical CPU count, the total memory in bytes — and the raw
/// bytes each came from, for the receipt's `facts` payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Facts {
    pub os: String,
    pub release: String,
    pub architecture: String,
    pub kernel: String,
    pub boot_id: String,
    pub logical_cpus: u32,
    pub memory_bytes: u64,
    /// The raw readings, in the order read: `os-release`, `version`, `boot_id`, `cpuinfo`, `meminfo`.
    pub raw: Vec<u8>,
}

/// Read every host fact from its file.
///
/// # Errors
/// `Io` for a file that cannot be read, `Bound` past [`MAX_FACT_BYTES`], `Missing` for a field a
/// file does not carry.
pub fn facts() -> Result<Facts, Error> {
    let os_release = bounded(Path::new("/etc/os-release"))?;
    let version = bounded(Path::new("/proc/version"))?;
    let boot_id = bounded(Path::new("/proc/sys/kernel/random/boot_id"))?;
    let cpuinfo = bounded(Path::new("/proc/cpuinfo"))?;
    let meminfo = bounded(Path::new("/proc/meminfo"))?;
    let (os, release) = os_release_fields(&os_release)?;
    let mut raw = Vec::new();
    for reading in [&os_release, &version, &boot_id, &cpuinfo, &meminfo] {
        raw.extend_from_slice(reading);
        raw.push(b'\n');
    }
    Ok(Facts {
        os,
        release,
        architecture: std::env::consts::ARCH.to_owned(),
        kernel: kernel_line(&version)?,
        boot_id: boot_id_line(&boot_id)?,
        logical_cpus: logical_cpus(&cpuinfo)?,
        memory_bytes: memory_bytes(&meminfo)?,
        raw,
    })
}

fn bounded(path: &Path) -> Result<Vec<u8>, Error> {
    let file = File::open(path).map_err(|_| Error::Io)?;
    let mut bytes = Vec::new();
    file.take(MAX_FACT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::Io)?;
    if bytes.len() > MAX_FACT_BYTES {
        return Err(Error::Bound);
    }
    Ok(bytes)
}

/// `ID` and `VERSION_ID` of an os-release file, unquoted.
///
/// # Errors
/// `Missing` when either key is absent.
pub fn os_release_fields(bytes: &[u8]) -> Result<(String, String), Error> {
    let text = String::from_utf8_lossy(bytes);
    let field = |key: &str| {
        text.lines()
            .find_map(|line| {
                line.strip_prefix(key)
                    .and_then(|rest| rest.strip_prefix('='))
            })
            .map(|value| value.trim().trim_matches('"').to_owned())
            .filter(|value| !value.is_empty())
            .ok_or(Error::Missing)
    };
    Ok((field("ID")?, field("VERSION_ID")?))
}

/// The kernel's release, the third word of `/proc/version` (`Linux version <release> ...`).
///
/// # Errors
/// `Missing` when the line has no third word.
pub fn kernel_line(bytes: &[u8]) -> Result<String, Error> {
    String::from_utf8_lossy(bytes)
        .split_whitespace()
        .nth(2)
        .map(str::to_owned)
        .ok_or(Error::Missing)
}

/// The boot id, trimmed: a UUID the kernel minted at boot.
///
/// # Errors
/// `Missing` when the file is empty or not a 36-byte id.
pub fn boot_id_line(bytes: &[u8]) -> Result<String, Error> {
    let id = String::from_utf8_lossy(bytes).trim().to_owned();
    if id.len() == 36 {
        Ok(id)
    } else {
        Err(Error::Missing)
    }
}

/// The logical CPU count: the `processor` lines of `/proc/cpuinfo`.
///
/// # Errors
/// `Missing` when there is none.
pub fn logical_cpus(bytes: &[u8]) -> Result<u32, Error> {
    let count = String::from_utf8_lossy(bytes)
        .lines()
        .filter(|line| line.starts_with("processor"))
        .count();
    u32::try_from(count)
        .ok()
        .filter(|count| *count > 0)
        .ok_or(Error::Missing)
}

/// Total memory in bytes: `MemTotal: <n> kB` of `/proc/meminfo`.
///
/// # Errors
/// `Missing` when the line is absent or not a kB count.
pub fn memory_bytes(bytes: &[u8]) -> Result<u64, Error> {
    String::from_utf8_lossy(bytes)
        .lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))
        .and_then(|rest| {
            let mut words = rest.split_whitespace();
            let value: u64 = words.next()?.parse().ok()?;
            (words.next() == Some("kB")).then_some(value.checked_mul(1024)?)
        })
        .ok_or(Error::Missing)
}

#[cfg(test)]
mod tests {
    use super::{Error, boot_id_line, kernel_line, logical_cpus, memory_bytes, os_release_fields};

    /// Each parser over this host's own readings (2026-09-26, Fedora 44 toolbox), and each refusal
    /// by name; every expected value is the file's, not the parser's.
    #[test]
    fn each_fact_is_parsed_from_its_file_and_refused_by_name() -> Result<(), Error> {
        let os_release = b"NAME=\"Fedora Linux\"\nVERSION=\"44 (Toolbx Container Image)\"\nRELEASE_TYPE=stable\nID=fedora\nVERSION_ID=44\nVERSION_CODENAME=\"\"\n";
        assert_eq!(
            os_release_fields(os_release)?,
            ("fedora".to_owned(), "44".to_owned())
        );
        assert_eq!(
            os_release_fields(b"NAME=x\nID=fedora\n"),
            Err(Error::Missing)
        );
        assert_eq!(
            os_release_fields(b"ID=\"\"\nVERSION_ID=44\n"),
            Err(Error::Missing)
        );
        let version = b"Linux version 7.2.5-200.fc44.x86_64 (mockbuild@b7cf753306404a0aace7b8416a1a73b3) (gcc (GCC) 16.2.1 20260819 (Red Hat 16.2.1-2), GNU ld version 2.46.1-1.fc44) #1 SMP PREEMPT_DYNAMIC Fri Sep 11 15:11:05 UTC 2026\n";
        assert_eq!(kernel_line(version)?, "7.2.5-200.fc44.x86_64");
        assert_eq!(kernel_line(b"Linux version\n"), Err(Error::Missing));
        assert_eq!(
            boot_id_line(b"270eb2e7-bf61-4a5c-9618-2c7e71817fb4\n")?,
            "270eb2e7-bf61-4a5c-9618-2c7e71817fb4"
        );
        assert_eq!(boot_id_line(b"\n"), Err(Error::Missing));
        assert_eq!(boot_id_line(b"270eb2e7\n"), Err(Error::Missing));
        let cpuinfo = b"processor\t: 0\nvendor_id\t: GenuineIntel\nprocessor\t: 1\nvendor_id\t: GenuineIntel\nprocessor\t: 2\n";
        assert_eq!(logical_cpus(cpuinfo)?, 3);
        assert_eq!(logical_cpus(b"vendor_id\t: x\n"), Err(Error::Missing));
        assert_eq!(
            memory_bytes(b"MemTotal:       98560948 kB\nMemFree:        12 kB\n")?,
            98_560_948 * 1024
        );
        assert_eq!(
            memory_bytes(b"MemTotal:       98560948 MB\n"),
            Err(Error::Missing)
        );
        assert_eq!(memory_bytes(b"MemFree: 12 kB\n"), Err(Error::Missing));
        Ok(())
    }
}
