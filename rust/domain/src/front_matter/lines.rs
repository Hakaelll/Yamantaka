use super::*;

pub(super) struct Line {
    pub text: String,
    pub bounds: NormalizedPageRect,
    pub layout: NormalizedPageRect,
}
impl Line {
    pub fn height(&self) -> f64 {
        self.layout.max()[1] - self.layout.min()[1]
    }
}

pub(super) struct PageAnalysis {
    pub lines: Vec<Line>,
    pub front_matter: bool,
}
impl PageAnalysis {
    pub fn prominent_title(&self) -> Option<&Line> {
        let mut heights: Vec<_> = self.lines.iter().map(Line::height).collect();
        heights.sort_by(f64::total_cmp);
        let typical = *heights.get(heights.len().saturating_sub(1) / 2)?;
        self.lines
            .iter()
            .filter(|line| {
                let top = line.layout.min()[1];
                top < 0.6
                    && line.height() >= 0.025
                    && line.height() >= typical * 1.5
                    && line.text.chars().count() <= 120
                    && line.text.split_whitespace().count() <= 16
                    && byline(&line.text).is_none()
                    && matches!(
                        import_metadata::title(Some(&line.text), ""),
                        MetadataValue::Accepted(_)
                    )
                    && !matches!(
                        normalized(&line.text).as_str(),
                        "contents" | "table of contents" | "índice" | "indice" | "copyright"
                    )
            })
            .min_by(
                |left, right| match (self.byline_gap(left), self.byline_gap(right)) {
                    (Some(left), Some(right)) => left.total_cmp(&right),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => right.height().total_cmp(&left.height()),
                },
            )
    }

    fn byline_gap(&self, title: &Line) -> Option<f64> {
        self.lines
            .iter()
            .filter_map(|line| {
                let value = byline(&line.text)?;
                if !matches!(
                    import_metadata::author(Some(value)),
                    MetadataValue::Accepted(_)
                ) || line.height() > title.height() * 0.8
                {
                    return None;
                }
                let gap = line.layout.min()[1] - title.layout.max()[1];
                (0.0..=0.16).contains(&gap).then_some(gap)
            })
            .min_by(f64::total_cmp)
    }

    pub fn explicit_authors<'a>(
        &'a self,
        title: Option<&'a Line>,
    ) -> impl Iterator<Item = &'a Line> {
        self.lines.iter().filter(move |line| {
            line.layout.min()[1] < 0.8
                && line.text.chars().count() <= 140
                && byline(&line.text).is_some_and(|value| !value.is_empty())
                && (title.is_some_and(|title| line.height() <= title.height() * 0.8)
                    || labeled_byline(&line.text))
        })
    }
}

pub(super) fn analyze(
    page: &PdfPageText,
    coordinates: PageCoordinateSpace,
    cancel: &Cancellation,
) -> Result<PageAnalysis> {
    let mut crop_min = [f64::INFINITY; 2];
    let mut crop_max = [f64::NEG_INFINITY; 2];
    for point in [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]] {
        let point = coordinates.pdf_point(point);
        for axis in 0..2 {
            crop_min[axis] = crop_min[axis].min(point[axis]);
            crop_max[axis] = crop_max[axis].max(point[axis]);
        }
    }
    let canonical = PageCoordinateSpace::new([crop_min[0], crop_max[1]], crop_max, crop_min)?;
    let mut lines = Vec::new();
    let mut text = String::new();
    let mut bounds: Option<NormalizedPageRect> = None;
    let mut layout: Option<NormalizedPageRect> = None;
    let mut overflow = false;
    let mut line_count = 0;
    for (index, character) in page.characters().iter().enumerate() {
        if index % 128 == 0 {
            cancel.check()?;
        }
        let value = &page.text()[character.bytes.start as usize..character.bytes.end as usize];
        if value == "\n" || value == "\r" {
            flush(
                &mut lines,
                &mut text,
                &mut bounds,
                &mut layout,
                &mut overflow,
                &mut line_count,
            );
            if line_count > 24 {
                cancel.check()?;
                return Ok(PageAnalysis {
                    lines,
                    front_matter: false,
                });
            }
            continue;
        }
        if text.len() + value.len() > 1024 {
            overflow = true;
        }
        if !overflow {
            text.push_str(value);
        }
        if let Some(region) = character.region {
            bounds = Some(union(bounds, region)?);
            let mut min = [f64::INFINITY; 2];
            let mut max = [f64::NEG_INFINITY; 2];
            for point in [
                region.min(),
                region.max(),
                [region.min()[0], region.max()[1]],
                [region.max()[0], region.min()[1]],
            ] {
                let point = coordinates.pdf_point(point);
                for axis in 0..2 {
                    min[axis] = min[axis].min(point[axis]);
                    max[axis] = max[axis].max(point[axis]);
                }
            }
            if let Some(region) =
                canonical.normalized_rectangle([min[0], min[1], max[0], max[1]])?
            {
                layout = Some(union(layout, region)?);
            }
        }
    }
    flush(
        &mut lines,
        &mut text,
        &mut bounds,
        &mut layout,
        &mut overflow,
        &mut line_count,
    );
    cancel.check()?;
    Ok(PageAnalysis {
        lines,
        front_matter: line_count <= 24,
    })
}

fn flush(
    lines: &mut Vec<Line>,
    text: &mut String,
    bounds: &mut Option<NormalizedPageRect>,
    layout: &mut Option<NormalizedPageRect>,
    overflow: &mut bool,
    count: &mut usize,
) {
    if !text.trim().is_empty() {
        *count += 1;
    }
    if !*overflow
        && lines.len() < 128
        && let (Some(bounds), Some(layout)) = (bounds.take(), layout.take())
    {
        let clean = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if !clean.is_empty() && clean.chars().count() <= 180 {
            lines.push(Line {
                text: clean,
                bounds,
                layout,
            });
        }
    }
    text.clear();
    *bounds = None;
    *layout = None;
    *overflow = false;
}

fn union(
    old: Option<NormalizedPageRect>,
    region: NormalizedPageRect,
) -> Result<NormalizedPageRect> {
    match old {
        None => Ok(region),
        Some(old) => NormalizedPageRect::new(
            [
                old.min()[0].min(region.min()[0]),
                old.min()[1].min(region.min()[1]),
            ],
            [
                old.max()[0].max(region.max()[0]),
                old.max()[1].max(region.max()[1]),
            ],
        ),
    }
}

pub(super) fn byline(text: &str) -> Option<&str> {
    let lower = text.to_lowercase();
    for prefix in [
        "written by ",
        "author: ",
        "autor: ",
        "autora: ",
        "by ",
        "por ",
    ] {
        if lower.starts_with(prefix) {
            return Some(text[prefix.len()..].trim());
        }
    }
    None
}

fn labeled_byline(text: &str) -> bool {
    let lower = text.to_lowercase();
    ["written by ", "author: ", "autor: ", "autora: "]
        .iter()
        .any(|prefix| lower.starts_with(prefix))
}
