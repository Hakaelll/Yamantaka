use crate::{Failure, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadingGoal {
    pages: Option<u32>,
    minutes: Option<u32>,
}
impl ReadingGoal {
    pub fn new(pages: Option<u32>, minutes: Option<u32>) -> Result<Self> {
        if pages.is_some_and(|value| !(1..=10_000).contains(&value))
            || minutes.is_some_and(|value| !(1..=1440).contains(&value))
        {
            return Err(Failure::Invalid(
                "Reading goals require 1..=10000 pages and 1..=1440 minutes".into(),
            ));
        }
        Ok(Self { pages, minutes })
    }
    pub fn from_legacy(pages: Option<i64>, minutes: Option<i64>) -> Self {
        Self {
            pages: pages.map(|value| value.clamp(1, 10_000) as u32),
            minutes: minutes.map(|value| value.clamp(1, 1440) as u32),
        }
    }
    pub fn pages(self) -> Option<u32> {
        self.pages
    }
    pub fn minutes(self) -> Option<u32> {
        self.minutes
    }
    pub fn is_empty(self) -> bool {
        self.pages.is_none() && self.minutes.is_none()
    }
}
