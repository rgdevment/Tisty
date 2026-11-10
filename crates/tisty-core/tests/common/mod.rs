use std::path::Path;

pub fn rewritten(at: &Path, mut hand: impl FnMut(&str, Vec<u8>) -> Vec<u8>) {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(at).unwrap()).unwrap();
    let mut kept: Vec<(String, Vec<u8>)> = Vec::new();
    for n in 0..zip.len() {
        let mut one = zip.by_index(n).unwrap();
        let named = one.name().unwrap().to_string();
        let mut body = Vec::new();
        std::io::Read::read_to_end(&mut one, &mut body).unwrap();
        kept.push((named, body));
    }

    let mut writing = zip::ZipWriter::new(std::fs::File::create(at).unwrap());
    for (named, body) in kept {
        writing
            .start_file(&named, zip::write::SimpleFileOptions::default())
            .unwrap();
        let body = hand(&named, body);
        std::io::Write::write_all(&mut writing, &body).unwrap();
    }
    writing.finish().unwrap();
}
