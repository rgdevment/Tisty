use std::io::Write;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

type Failed = Box<dyn std::error::Error>;

const DIMS: usize = 256;
const PUBLISHED: &str = "huggingface.co/sentence-transformers/static-similarity-mrl-multilingual-v1 at revision b68f4122911bcffcd6e1f695f2d99cd6788972d8";
const PUBLISHED_SHA256: &str = "8245ab78ee71dded845a82d2270fcb9e785b29dad0e1619f69d5390c47d9ba00";

fn main() -> Result<(), Failed> {
    let mut args = std::env::args().skip(1).map(PathBuf::from);
    let (Some(model), Some(out)) = (args.next(), args.next()) else {
        return Err(format!(
            "usage: derive <0_StaticEmbedding folder of {PUBLISHED}> <out folder>"
        )
        .into());
    };
    std::fs::create_dir_all(&out)?;
    table(&model.join("model.safetensors"), &out.join("table.bin"))?;
    vocab(&model.join("tokenizer.json"), &out.join("vocab.txt"))?;
    Ok(())
}

// The model is trained so its first dimensions stand alone; each row keeps one scale for its bytes.
fn table(from: &Path, to: &Path) -> Result<(), Failed> {
    let bytes = std::fs::read(from)?;
    let sha: String = Sha256::digest(&bytes)
        .iter()
        .map(|one| format!("{one:02x}"))
        .collect();
    if sha != PUBLISHED_SHA256 {
        return Err(format!("model.safetensors is not the one of {PUBLISHED}").into());
    }
    let head = u64::from_le_bytes(bytes[0..8].try_into()?) as usize;
    let header: serde_json::Value = serde_json::from_slice(&bytes[8..8 + head])?;
    let (_, matrix) = header
        .as_object()
        .ok_or("no header")?
        .iter()
        .find(|(name, _)| *name != "__metadata__")
        .ok_or("no matrix")?;
    if matrix["dtype"] != "F32" {
        return Err("the matrix is not F32".into());
    }
    let shape: Vec<usize> = serde_json::from_value(matrix["shape"].clone())?;
    let offsets: Vec<usize> = serde_json::from_value(matrix["data_offsets"].clone())?;
    let (rows, wide) = (shape[0], shape[1]);
    let raw = &bytes[8 + head + offsets[0]..8 + head + offsets[1]];
    let value = |row: usize, col: usize| {
        let at = (row * wide + col) * 4;
        f32::from_le_bytes([raw[at], raw[at + 1], raw[at + 2], raw[at + 3]])
    };
    let mut file = std::io::BufWriter::new(std::fs::File::create(to)?);
    file.write_all(&u32::try_from(rows)?.to_le_bytes())?;
    file.write_all(&u32::try_from(DIMS)?.to_le_bytes())?;
    let mut bytes_of_rows = Vec::with_capacity(rows * DIMS);
    for row in 0..rows {
        let most = (0..DIMS)
            .map(|col| value(row, col).abs())
            .fold(0f32, f32::max)
            .max(1e-12);
        file.write_all(&(most / 127.0).to_le_bytes())?;
        bytes_of_rows
            .extend((0..DIMS).map(|col| ((value(row, col) / most) * 127.0).round() as i8 as u8));
    }
    file.write_all(&bytes_of_rows)?;
    Ok(())
}

fn vocab(from: &Path, to: &Path) -> Result<(), Failed> {
    let said: serde_json::Value = serde_json::from_slice(&std::fs::read(from)?)?;
    let pieces = said["model"]["vocab"].as_object().ok_or("no vocab")?;
    let mut by_id = vec![None; pieces.len()];
    for (piece, id) in pieces {
        let id = usize::try_from(id.as_u64().ok_or("an id is not a number")?)?;
        *by_id.get_mut(id).ok_or("the ids are not 0 to n-1")? = Some(piece.as_str());
    }
    let mut lines = Vec::with_capacity(by_id.len());
    for piece in by_id {
        let piece = piece.ok_or("an id is missing")?;
        if piece.contains('\n') {
            return Err("a piece holds a line break".into());
        }
        lines.push(piece);
    }
    std::fs::write(to, lines.join("\n"))?;
    Ok(())
}
