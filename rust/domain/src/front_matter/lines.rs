use super::*;

#[derive(Clone)]
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

pub(super) struct TitleBlock {
    pub line: Line,
    pub height: f64,
}

fn title_text(text: &str) -> bool {
    byline(text).is_none()
        && matches!(
            import_metadata::title(Some(text), ""),
            MetadataValue::Accepted(_)
        )
        && !matches!(
            normalized(text).as_str(),
            "contents"
                | "table of contents"
                | "índice"
                | "indice"
                | "copyright"
                | "publication statement"
        )
}

fn continues(previous: &Line, next: &Line) -> bool {
    let height = previous.height().max(next.height());
    let gap = next.layout.min()[1] - previous.layout.max()[1];
    let center = |line: &Line| (line.layout.min()[0] + line.layout.max()[0]) * 0.5;
    previous.height().min(next.height()) >= height * 0.6
        && (-height * 0.15..=height * 1.25).contains(&gap)
        && ((center(previous) - center(next)).abs() <= 0.06
            || (previous.layout.min()[0] - next.layout.min()[0]).abs() <= 0.025)
}

impl PageAnalysis {
    pub fn prominent_title(&self) -> Option<TitleBlock> {
        let mut heights: Vec<_> = self.lines.iter().map(Line::height).collect();
        heights.sort_by(f64::total_cmp);
        let typical = *heights.get(heights.len().saturating_sub(1) / 2)?;
        let mut blocks = Vec::new();
        let mut index = 0;
        while index < self.lines.len() {
            let first = &self.lines[index];
            index += 1;
            if first.layout.min()[1] >= 0.6 || first.height() < 0.018 || !title_text(&first.text) {
                continue;
            }
            let mut line = first.clone();
            let mut previous = first;
            let mut count = 1;
            let mut height = first.height();
            while count < 4 && index < self.lines.len() {
                let next = &self.lines[index];
                if next.layout.max()[1] > 0.6
                    || !title_text(&next.text)
                    || !continues(previous, next)
                    || line.text.chars().count() + next.text.chars().count() + 1 > 240
                {
                    break;
                }
                line.text.push(' ');
                line.text.push_str(&next.text);
                line.bounds = union(Some(line.bounds), next.bounds).ok()?;
                line.layout = union(Some(line.layout), next.layout).ok()?;
                height += next.height();
                count += 1;
                index += 1;
                previous = next;
            }
            height /= f64::from(count);
            let prominent = height >= 0.025 && height >= typical * 1.5;
            let sparse_block = count >= 2
                && self.lines.len() <= 12
                && first.layout.min()[1] < 0.35
                && height >= typical * 0.6;
            if (prominent || sparse_block) && line.text.split_whitespace().count() <= 40 {
                blocks.push(TitleBlock { line, height });
            }
        }
        blocks.into_iter().min_by(|left, right| {
            match (self.byline_gap(&left.line), self.byline_gap(&right.line)) {
                (Some(left), Some(right)) => left.total_cmp(&right),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => right.height.total_cmp(&left.height),
            }
        })
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
            line.layout.min()[1]
                < if labeled_byline(&line.text) {
                    0.94
                } else {
                    0.8
                }
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
        "compiled by ",
        "compilado por ",
        "compilada por ",
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
    [
        "written by ",
        "compiled by ",
        "compilado por ",
        "compilada por ",
        "author: ",
        "autor: ",
        "autora: ",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
}
