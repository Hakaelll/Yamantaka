use super::*;

pub struct AnnotationMark {
    id: String,
    kind: AnnotationKind,
    color: AnnotationColor,
    rects: Vec<AnnotationRect>,
    drawing: Option<StudyGeometry>,
    has_note: bool,
    _lease: ByteLease,
}
impl AnnotationMark {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn has_note(&self) -> bool {
        self.has_note
    }
    pub fn drawing(&self) -> Option<&StudyGeometry> {
        self.drawing.as_ref()
    }
    pub fn kind(&self) -> AnnotationKind {
        self.kind
    }
    pub fn color(&self) -> AnnotationColor {
        self.color
    }
    pub fn rects(&self) -> &[AnnotationRect] {
        &self.rects
    }
    pub fn from_reserved(
        id: String,
        kind: AnnotationKind,
        color: AnnotationColor,
        rects: Vec<AnnotationRect>,
        drawing: Option<StudyGeometry>,
        has_note: bool,
        mut lease: ByteLease,
    ) -> Result<Self> {
        let bytes = rects
            .capacity()
            .checked_mul(std::mem::size_of::<AnnotationRect>())
            .and_then(|bytes| bytes.checked_add(id.capacity()))
            .and_then(|bytes| {
                bytes.checked_add(drawing.as_ref().map_or(0, StudyGeometry::allocated_bytes))
            })
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>()))
            .ok_or(Failure::MemoryBudget)?;
        lease.try_resize(bytes).map_err(|_| Failure::MemoryBudget)?;
        Ok(Self {
            id,
            kind,
            color,
            rects,
            drawing,
            has_note,
            _lease: lease,
        })
    }
}
pub struct AnnotationMarks {
    page: u32,
    marks: Vec<AnnotationMark>,
    _lease: ByteLease,
}
impl AnnotationMarks {
    pub fn page(&self) -> u32 {
        self.page
    }
    pub fn marks(&self) -> &[AnnotationMark] {
        &self.marks
    }
    pub fn from_reserved(
        page: u32,
        marks: Vec<AnnotationMark>,
        mut lease: ByteLease,
    ) -> Result<Self> {
        let bytes = marks
            .capacity()
            .checked_mul(std::mem::size_of::<AnnotationMark>())
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>()))
            .ok_or(Failure::MemoryBudget)?;
        lease.try_resize(bytes).map_err(|_| Failure::MemoryBudget)?;
        Ok(Self {
            page,
            marks,
            _lease: lease,
        })
    }
}
