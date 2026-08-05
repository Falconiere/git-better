const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0000_0100_0000_01b3;

/// Returns the FNV-1a 64-bit digest of `bytes` as 16 lowercase hex characters.
///
/// Cache keys and change detection need a stable, dependency-free digest, not a
/// cryptographic one; this definition never drifts with the toolchain.
pub fn fnv1a_hex(bytes: &[u8]) -> String {
  let mut acc = OFFSET_BASIS;
  for byte in bytes {
    acc ^= u64::from(*byte);
    acc = acc.wrapping_mul(PRIME);
  }
  format!("{acc:016x}")
}
