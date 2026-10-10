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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_targets_are_strict_and_legacy_integer_targets_are_clamped() {
        for value in [0, 10001, u32::MAX] {
            assert!(ReadingGoal::new(Some(value), None).is_err());
        }
        for value in [0, 1441, u32::MAX] {
            assert!(ReadingGoal::new(None, Some(value)).is_err());
        }
        assert!(ReadingGoal::new(None, None).unwrap().is_empty());
        assert_eq!(
            ReadingGoal::from_legacy(Some(i64::MIN), Some(i64::MAX)),
            ReadingGoal::new(Some(1), Some(1440)).unwrap()
        );
    }
}
