use crate::{Document, Failure, Result, annotations::AnnotationRect};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const KEY: &str = "rust.studyLibrary.v1";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Library {
    pub revision: u64,
    pub notebooks: Vec<Notebook>,
    pub links: Vec<PassageLink>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Notebook {
    pub title: String,
    pub entries: Vec<Passage>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Passage {
    pub document: String,
    pub fingerprint: String,
    pub title: String,
    pub page: u32,
    pub last_page: u32,
    pub point: [f64; 2],
    pub rects: Vec<AnnotationRect>,
    pub text: String,
    pub annotation: Option<String>,
    pub crop: Option<[[f64; 2]; 2]>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PassageLink {
    pub passages: [Passage; 2],
    pub note: String,
}

pub fn decode(json: &str) -> Result<Library> {
    if json.len() > 1_000_000 {
        return Err(Failure::Invalid(
            "Study library exceeds the storage limit".into(),
        ));
    }
    let library: Library =
        serde_json::from_str(json).map_err(|_| Failure::Invalid("Invalid study library".into()))?;
    let point = |point: &[f64; 2]| point.iter().all(|n| (0.0..=1.0).contains(n));
    let passage = |p: &Passage| {
        !p.document.is_empty()
            && p.document.len() <= 256
            && !p.fingerprint.is_empty()
            && p.fingerprint.len() <= 256
            && p.title.len() <= 2048
            && p.last_page >= p.page
            && p.last_page < i32::MAX as u32
            && p.text.len() <= 16_384
            && p.rects.len() <= 128
            && point(&p.point)
            && p.annotation
                .as_ref()
                .is_none_or(|id| !id.is_empty() && id.len() <= 256)
            && p.crop
                .as_ref()
                .is_none_or(|[a, b]| point(a) && point(b) && a[0] != b[0] && a[1] != b[1])
    };
    if library.revision >= i64::MAX as u64
        || library.notebooks.len() > 40
        || library.links.len() > 200
        || !library.notebooks.iter().all(|n| {
            !n.title.trim().is_empty()
                && n.title.len() <= 300
                && !n.title.contains(['\r', '\n'])
                && n.entries.len() <= 200
                && n.entries.iter().all(passage)
        })
        || !library
            .links
            .iter()
            .all(|l| l.note.len() <= 8192 && l.passages.iter().all(passage))
    {
        return Err(Failure::Invalid(
            "Invalid notebook entries or passage links".into(),
        ));
    }
    Ok(library)
}

pub enum Operation {
    Load,
    Save(Library),
    Export(usize, PathBuf),
    Resolve(Vec<Passage>),
}

pub enum Outcome {
    Library(Library),
    Exported,
    Resolved(Vec<(Passage, Document)>),
}

pub struct PdfMarks {
    pub document: String,
    pub fingerprint: String,
    pub pages: std::ops::Range<u32>,
    pub destination: PathBuf,
}

pub struct PdfMark {
    pub page: u32,
    pub kind: crate::annotations::AnnotationKind,
    pub color: crate::annotations::AnnotationColor,
    pub rects: Vec<AnnotationRect>,
    pub drawing: Option<crate::annotations::study::StudyGeometry>,
}
