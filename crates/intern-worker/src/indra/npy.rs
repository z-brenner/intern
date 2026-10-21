//! NumPy `.npy` (format 1.0/2.0, little-endian, C order) and `.npz` files:
//! the language models are `.npz` archives, and the parity tests read the
//! reference package's dumped tensors as `.npy`.

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

/// The element data of an array.
#[derive(Clone, Debug, PartialEq)]
pub enum NpyData {
    Bool(Vec<bool>),
    U8(Vec<u8>),
    I8(Vec<i8>),
    I32(Vec<i32>),
    I64(Vec<i64>),
    U64(Vec<u64>),
    F32(Vec<f32>),
    F64(Vec<f64>),
}

/// An array: its shape (empty for a scalar) and its elements in C order.
#[derive(Clone, Debug, PartialEq)]
pub struct NpyArray {
    pub shape: Vec<usize>,
    pub data: NpyData,
}

#[derive(Debug, thiserror::Error)]
pub enum NpyError {
    #[error("not an .npy file")]
    Magic,
    #[error("unsupported .npy header: {0}")]
    Header(String),
    #[error(".npy data is {found} bytes, expected {expected}")]
    Length { found: usize, expected: usize },
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Zip(#[from] zip::result::ZipError),
}

impl NpyArray {
    pub fn len(&self) -> usize {
        self.shape.iter().product()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_f32(&self) -> Option<&[f32]> {
        match &self.data {
            NpyData::F32(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<&[u64]> {
        match &self.data {
            NpyData::U64(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<&[i64]> {
        match &self.data {
            NpyData::I64(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> Option<&[u8]> {
        match &self.data {
            NpyData::U8(v) => Some(v),
            _ => None,
        }
    }

    /// A scalar or one-element integer array as an i64, whatever its width.
    pub fn scalar_i64(&self) -> Option<i64> {
        if self.len() != 1 {
            return None;
        }
        Some(match &self.data {
            NpyData::U8(v) => i64::from(v[0]),
            NpyData::I8(v) => i64::from(v[0]),
            NpyData::I32(v) => i64::from(v[0]),
            NpyData::I64(v) => v[0],
            NpyData::U64(v) => i64::try_from(v[0]).ok()?,
            _ => return None,
        })
    }
}

/// Parses one `.npy` file's bytes.
pub fn parse(bytes: &[u8]) -> Result<NpyArray, NpyError> {
    if bytes.len() < 10 || &bytes[..6] != b"\x93NUMPY" {
        return Err(NpyError::Magic);
    }
    let major = bytes[6];
    let (header_len, start) = match major {
        1 => (usize::from(u16::from_le_bytes([bytes[8], bytes[9]])), 10),
        2 | 3 => {
            if bytes.len() < 12 {
                return Err(NpyError::Magic);
            }
            let len = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
            (len as usize, 12)
        }
        _ => return Err(NpyError::Header(format!("version {major}"))),
    };
    let header = bytes
        .get(start..start + header_len)
        .ok_or_else(|| NpyError::Header("truncated".into()))?;
    let header = std::str::from_utf8(header).map_err(|_| NpyError::Header("not text".into()))?;
    let descr = dict_value(header, "descr").ok_or_else(|| NpyError::Header(header.into()))?;
    let fortran = dict_value(header, "fortran_order").unwrap_or("False");
    if fortran.trim() != "False" {
        return Err(NpyError::Header("Fortran order".into()));
    }
    let shape = parse_shape(header).ok_or_else(|| NpyError::Header(header.into()))?;
    let descr = descr.trim().trim_matches(|c| c == '\'' || c == '"');
    let body = &bytes[start + header_len..];
    let count: usize = shape.iter().product();
    let width = match descr {
        "|b1" | "|u1" | "|i1" => 1,
        "<i4" => 4,
        "<i8" | "<u8" | "<f8" => 8,
        "<f4" => 4,
        other => return Err(NpyError::Header(format!("dtype {other}"))),
    };
    let expected = count * width;
    if body.len() < expected {
        return Err(NpyError::Length {
            found: body.len(),
            expected,
        });
    }
    let body = &body[..expected];
    let chunks = |n: usize| body.chunks_exact(n);
    let data = match descr {
        "|b1" => NpyData::Bool(body.iter().map(|&b| b != 0).collect()),
        "|u1" => NpyData::U8(body.to_vec()),
        "|i1" => NpyData::I8(body.iter().map(|&b| b as i8).collect()),
        "<i4" => NpyData::I32(
            chunks(4)
                .map(|c| i32::from_le_bytes(c.try_into().unwrap()))
                .collect(),
        ),
        "<i8" => NpyData::I64(
            chunks(8)
                .map(|c| i64::from_le_bytes(c.try_into().unwrap()))
                .collect(),
        ),
        "<u8" => NpyData::U64(
            chunks(8)
                .map(|c| u64::from_le_bytes(c.try_into().unwrap()))
                .collect(),
        ),
        "<f4" => NpyData::F32(
            chunks(4)
                .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
                .collect(),
        ),
        "<f8" => NpyData::F64(
            chunks(8)
                .map(|c| f64::from_le_bytes(c.try_into().unwrap()))
                .collect(),
        ),
        _ => unreachable!(),
    };
    Ok(NpyArray { shape, data })
}

/// Reads one `.npy` file.
pub fn read(path: &Path) -> Result<NpyArray, NpyError> {
    parse(&std::fs::read(path)?)
}

/// Reads every array of an `.npz` archive, keyed by name without `.npy`.
pub fn read_npz(path: &Path) -> Result<HashMap<String, NpyArray>, NpyError> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))?;
    let mut arrays = HashMap::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().trim_end_matches(".npy").to_owned();
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut bytes)?;
        arrays.insert(name, parse(&bytes)?);
    }
    Ok(arrays)
}

fn dict_value<'a>(header: &'a str, key: &str) -> Option<&'a str> {
    let at = header
        .find(&format!("'{key}'"))
        .or_else(|| header.find(&format!("\"{key}\"")))?;
    let rest = &header[at + key.len() + 2..];
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let end = rest.find([',', '}']).unwrap_or(rest.len());
    Some(&rest[..end])
}

fn parse_shape(header: &str) -> Option<Vec<usize>> {
    let at = header
        .find("'shape'")
        .or_else(|| header.find("\"shape\""))?;
    let rest = &header[at..];
    let open = rest.find('(')?;
    let close = rest[open..].find(')')? + open;
    rest[open + 1..close]
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_end_matches('L').parse().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn npy(descr: &str, shape: &str, body: &[u8]) -> Vec<u8> {
        let mut header =
            format!("{{'descr': '{descr}', 'fortran_order': False, 'shape': {shape}, }}");
        while (10 + header.len() + 1) % 64 != 0 {
            header.push(' ');
        }
        header.push('\n');
        let mut out = b"\x93NUMPY\x01\x00".to_vec();
        out.extend_from_slice(&(header.len() as u16).to_le_bytes());
        out.extend_from_slice(header.as_bytes());
        out.extend_from_slice(body);
        out
    }

    #[test]
    fn reads_shapes_and_dtypes() {
        let body: Vec<u8> = [1.5f32, -2.0, 3.25, 0.0]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let array = parse(&npy("<f4", "(2, 2)", &body)).unwrap();
        assert_eq!(array.shape, vec![2, 2]);
        assert_eq!(array.as_f32().unwrap(), &[1.5, -2.0, 3.25, 0.0]);

        let scalar = parse(&npy("|i1", "()", &[5])).unwrap();
        assert!(scalar.shape.is_empty());
        assert_eq!(scalar.scalar_i64(), Some(5));

        let one = parse(&npy("<u8", "(1,)", &7u64.to_le_bytes())).unwrap();
        assert_eq!(one.shape, vec![1]);
        assert_eq!(one.as_u64().unwrap(), &[7]);
    }

    #[test]
    fn refuses_truncated_data_and_fortran_order() {
        assert!(matches!(
            parse(&npy("<f4", "(3,)", &[0; 8])),
            Err(NpyError::Length { .. })
        ));
        let mut fortran = npy("<f4", "(1,)", &[0; 4]);
        let at = fortran.windows(5).position(|w| w == b"False").unwrap();
        fortran[at..at + 5].copy_from_slice(b"True ");
        assert!(matches!(parse(&fortran), Err(NpyError::Header(_))));
        assert!(matches!(parse(b"not numpy"), Err(NpyError::Magic)));
    }
}
