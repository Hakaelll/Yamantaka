use super::*;

pub struct AnnotationMark {
    kind: AnnotationKind,
    color: AnnotationColor,
    rects: Vec<AnnotationRect>,
    _lease: ByteLease,
}
impl AnnotationMark {
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
        kind: AnnotationKind,
        color: AnnotationColor,
        rects: Vec<AnnotationRect>,
        mut lease: ByteLease,
    ) -> Result<Self> {
        let bytes = rects
            .capacity()
            .checked_mul(std::mem::size_of::<AnnotationRect>())
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>()))
            .ok_or(Failure::MemoryBudget)?;
        lease.try_resize(bytes).map_err(|_| Failure::MemoryBudget)?;
        Ok(Self {
            kind,
            color,
            rects,
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
