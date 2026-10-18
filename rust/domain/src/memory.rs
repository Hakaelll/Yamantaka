//! Admission and lifetime accounting for resources controlled by the application.
//!
//! Categories and representations are two views of the same bytes: a CPU page
//! contributes once to the total, once to Pages, and once to Cpu. Native library
//! and driver allocations which are not exposed to the application are excluded.
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

const MIB: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryCategory {
    Pages,
    Tiles,
    Text,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRepresentation {
    Cpu,
    Gpu,
    Transient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cost {
    pub category: MemoryCategory,
    pub representation: MemoryRepresentation,
    pub bytes: usize,
}

impl Cost {
    pub const fn new(
        category: MemoryCategory,
        representation: MemoryRepresentation,
        bytes: usize,
    ) -> Self {
        Self {
            category,
            representation,
            bytes,
        }
    }
}

/// All limits apply globally, including resources retained outside caches.
/// Reducing limits changes admission; it never revokes an existing lease.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalMemoryLimits {
    pub total: usize,
    pub pages: usize,
    pub tiles: usize,
    pub text: usize,
    pub other: Option<usize>,
    pub cpu: Option<usize>,
    pub gpu: Option<usize>,
    pub transient: Option<usize>,
}

impl Default for GlobalMemoryLimits {
    fn default() -> Self {
        Self {
            total: 104 * MIB,
            pages: 72 * MIB,
            tiles: 24 * MIB,
            text: 8 * MIB,
            other: None,
            cpu: None,
            gpu: None,
            transient: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryScope {
    Total,
    Category(MemoryCategory),
    Representation(MemoryRepresentation),
}

#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Memory accounting overflow")]
    Overflow,
    #[error(
        "Memory budget denied for {scope:?}: {used} live + {requested} requested exceeds {limit}"
    )]
    BudgetDenied {
        scope: MemoryScope,
        used: usize,
        requested: usize,
        limit: usize,
    },
    #[error("Memory allocation failed: {0}")]
    AllocationFailed(#[from] std::collections::TryReserveError),
    #[error("Byte storage cannot use a GPU-only lease")]
    InvalidByteRepresentation,
}

pub type MemoryResult<T> = std::result::Result<T, MemoryError>;

/// Category and representation counters each sum to `total`, independently.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoryUsage {
    pub total: usize,
    pub pages: usize,
    pub tiles: usize,
    pub text: usize,
    pub other: usize,
    pub cpu: usize,
    pub gpu: usize,
    pub transient: usize,
}

impl MemoryUsage {
    pub fn category(self, category: MemoryCategory) -> usize {
        match category {
            MemoryCategory::Pages => self.pages,
            MemoryCategory::Tiles => self.tiles,
            MemoryCategory::Text => self.text,
            MemoryCategory::Other => self.other,
        }
    }

    pub fn representation(self, representation: MemoryRepresentation) -> usize {
        match representation {
            MemoryRepresentation::Cpu => self.cpu,
            MemoryRepresentation::Gpu => self.gpu,
            MemoryRepresentation::Transient => self.transient,
        }
    }

    fn category_mut(&mut self, category: MemoryCategory) -> &mut usize {
        match category {
            MemoryCategory::Pages => &mut self.pages,
            MemoryCategory::Tiles => &mut self.tiles,
            MemoryCategory::Text => &mut self.text,
            MemoryCategory::Other => &mut self.other,
        }
    }

    fn representation_mut(&mut self, representation: MemoryRepresentation) -> &mut usize {
        match representation {
            MemoryRepresentation::Cpu => &mut self.cpu,
            MemoryRepresentation::Gpu => &mut self.gpu,
            MemoryRepresentation::Transient => &mut self.transient,
        }
    }

    fn checked_add_cost(&mut self, cost: Cost) -> MemoryResult<()> {
        self.total = self
            .total
            .checked_add(cost.bytes)
            .ok_or(MemoryError::Overflow)?;
        let category = self.category_mut(cost.category);
        *category = category
            .checked_add(cost.bytes)
            .ok_or(MemoryError::Overflow)?;
        let representation = self.representation_mut(cost.representation);
        *representation = representation
            .checked_add(cost.bytes)
            .ok_or(MemoryError::Overflow)?;
        Ok(())
    }

    fn subtract_cost(&mut self, cost: Cost) {
        // Costs can only be released by their unique, non-clonable lease.
        self.total -= cost.bytes;
        *self.category_mut(cost.category) -= cost.bytes;
        *self.representation_mut(cost.representation) -= cost.bytes;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemorySnapshot {
    pub limits: GlobalMemoryLimits,
    pub usage: MemoryUsage,
    pub capacity_epoch: u64,
    /// True when a configuration decrease leaves existing leases over budget.
    pub under_pressure: bool,
}

#[derive(Debug)]
struct State {
    limits: GlobalMemoryLimits,
    usage: MemoryUsage,
    capacity_epoch: u64,
}

#[derive(Debug)]
struct Inner {
    state: Mutex<State>,
    capacity_changed: Condvar,
}

/// Clones share a single ledger. This type may also be held in an `Arc`.
#[derive(Debug, Clone)]
pub struct MemoryGovernor {
    inner: Arc<Inner>,
}

impl MemoryGovernor {
    pub fn new(limits: GlobalMemoryLimits) -> Self {
        Self {
            inner: Arc::new(Inner {
                state: Mutex::new(State {
                    limits,
                    usage: MemoryUsage::default(),
                    capacity_epoch: 0,
                }),
                capacity_changed: Condvar::new(),
            }),
        }
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        // All accounting operations are checked before committing and invoke no
        // caller code under the lock. Recovering poison keeps RAII drop usable.
        self.inner
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    pub fn snapshot(&self) -> MemorySnapshot {
        let state = self.lock();
        MemorySnapshot {
            limits: state.limits,
            usage: state.usage,
            capacity_epoch: state.capacity_epoch,
            under_pressure: check_limits(state.limits, state.usage, MemoryUsage::default())
                .is_err(),
        }
    }

    pub fn usage(&self) -> MemoryUsage {
        self.lock().usage
    }

    pub fn limits(&self) -> GlobalMemoryLimits {
        self.lock().limits
    }

    /// Signals capacity changes even if the new limits still cause pressure.
    pub fn set_limits(&self, limits: GlobalMemoryLimits) {
        let mut state = self.lock();
        if state.limits == limits {
            return;
        }
        state.limits = limits;
        state.capacity_epoch = state.capacity_epoch.wrapping_add(1);
        drop(state);
        self.inner.capacity_changed.notify_all();
    }

    pub fn capacity_epoch(&self) -> u64 {
        self.lock().capacity_epoch
    }

    /// Waits without polling. Capture the epoch before a rejected admission and
    /// pass it here, so a release between rejection and waiting is not missed.
    /// The returned epoch equals `observed` only if the timeout elapsed.
    pub fn wait_for_capacity_change(&self, observed: u64, timeout: Duration) -> u64 {
        let (state, _) = self
            .inner
            .capacity_changed
            .wait_timeout_while(self.lock(), timeout, |state| {
                state.capacity_epoch == observed
            })
            .unwrap_or_else(|error| error.into_inner());
        state.capacity_epoch
    }

    /// Reserves every cost or none. Allocates lease metadata before committing.
    pub fn try_reserve(&self, costs: &[Cost]) -> MemoryResult<ReservationSet> {
        let mut leases = Vec::new();
        leases.try_reserve_exact(costs.len())?;
        let mut requested = MemoryUsage::default();
        for &cost in costs {
            requested.checked_add_cost(cost)?;
        }
        {
            let mut state = self.lock();
            check_limits(state.limits, state.usage, requested)?;
            let mut next = state.usage;
            for &cost in costs {
                next.checked_add_cost(cost)?;
            }
            state.usage = next;
        }
        for &cost in costs {
            leases.push(ByteLease {
                governor: self.clone(),
                cost,
            });
        }
        Ok(ReservationSet { leases })
    }

    pub fn try_reserve_one(&self, cost: Cost) -> MemoryResult<ByteLease> {
        // A single lease needs no metadata allocation.
        let mut state = self.lock();
        let mut requested = MemoryUsage::default();
        requested.checked_add_cost(cost)?;
        check_limits(state.limits, state.usage, requested)?;
        let mut next = state.usage;
        next.checked_add_cost(cost)?;
        state.usage = next;
        Ok(ByteLease {
            governor: self.clone(),
            cost,
        })
    }

    fn release(&self, cost: Cost) {
        if cost.bytes == 0 {
            return;
        }
        let mut state = self.lock();
        state.usage.subtract_cost(cost);
        state.capacity_epoch = state.capacity_epoch.wrapping_add(1);
        drop(state);
        self.inner.capacity_changed.notify_all();
    }
}

impl Default for MemoryGovernor {
    fn default() -> Self {
        Self::new(GlobalMemoryLimits::default())
    }
}

fn check_limit(
    scope: MemoryScope,
    used: usize,
    requested: usize,
    limit: usize,
) -> MemoryResult<()> {
    let next = used.checked_add(requested).ok_or(MemoryError::Overflow)?;
    if next > limit {
        return Err(MemoryError::BudgetDenied {
            scope,
            used,
            requested,
            limit,
        });
    }
    Ok(())
}

fn check_limits(
    limits: GlobalMemoryLimits,
    used: MemoryUsage,
    requested: MemoryUsage,
) -> MemoryResult<()> {
    check_limit(
        MemoryScope::Total,
        used.total,
        requested.total,
        limits.total,
    )?;
    for (category, limit) in [
        (MemoryCategory::Pages, Some(limits.pages)),
        (MemoryCategory::Tiles, Some(limits.tiles)),
        (MemoryCategory::Text, Some(limits.text)),
        (MemoryCategory::Other, limits.other),
    ] {
        if let Some(limit) = limit {
            check_limit(
                MemoryScope::Category(category),
                used.category(category),
                requested.category(category),
                limit,
            )?;
        }
    }
    for (representation, limit) in [
        (MemoryRepresentation::Cpu, limits.cpu),
        (MemoryRepresentation::Gpu, limits.gpu),
        (MemoryRepresentation::Transient, limits.transient),
    ] {
        if let Some(limit) = limit {
            check_limit(
                MemoryScope::Representation(representation),
                used.representation(representation),
                requested.representation(representation),
                limit,
            )?;
        }
    }
    Ok(())
}

/// Unique accounting ownership; move this with the corresponding resource.
/// Retain it until physical retirement finishes for asynchronously freed GPU data.
#[derive(Debug)]
pub struct ByteLease {
    governor: MemoryGovernor,
    cost: Cost,
}

impl ByteLease {
    pub fn cost(&self) -> Cost {
        self.cost
    }

    pub fn bytes(&self) -> usize {
        self.cost.bytes
    }

    /// Reconciles an allocator's actual capacity. An increase is admitted
    /// atomically; a decrease is always allowed, including under pressure.
    pub fn try_resize(&mut self, bytes: usize) -> MemoryResult<()> {
        if bytes > self.cost.bytes {
            let additional = Cost {
                bytes: bytes - self.cost.bytes,
                ..self.cost
            };
            let mut state = self.governor.lock();
            let mut requested = MemoryUsage::default();
            requested.checked_add_cost(additional)?;
            check_limits(state.limits, state.usage, requested)?;
            let mut next = state.usage;
            next.checked_add_cost(additional)?;
            state.usage = next;
            self.cost.bytes = bytes;
        } else if bytes < self.cost.bytes {
            let released = Cost {
                bytes: self.cost.bytes - bytes,
                ..self.cost
            };
            self.cost.bytes = bytes;
            self.governor.release(released);
        }
        Ok(())
    }
}

impl Drop for ByteLease {
    fn drop(&mut self) {
        self.governor.release(self.cost);
    }
}

/// Unique group reservation. Moving individual leases does not change accounting.
#[derive(Debug)]
pub struct ReservationSet {
    leases: Vec<ByteLease>,
}

impl ReservationSet {
    pub fn len(&self) -> usize {
        self.leases.len()
    }

    pub fn is_empty(&self) -> bool {
        self.leases.is_empty()
    }

    pub fn leases(&self) -> &[ByteLease] {
        &self.leases
    }

    /// Removes the last cost in the order passed to `try_reserve`.
    pub fn pop(&mut self) -> Option<ByteLease> {
        self.leases.pop()
    }

    pub fn into_leases(self) -> Vec<ByteLease> {
        self.leases
    }
}

/// Immutable bytes plus a lease for actual allocated capacity, not just length.
/// Sharing this value through `Arc` retains the accounting until the last drop.
#[derive(Debug)]
pub struct TrackedBytes {
    // Field order releases the allocation before its accounting lease.
    bytes: Vec<u8>,
    lease: ByteLease,
}

impl TrackedBytes {
    pub fn try_zeroed(
        governor: &MemoryGovernor,
        category: MemoryCategory,
        len: usize,
    ) -> MemoryResult<Self> {
        Ok(TrackedBytesBuilder::try_zeroed(governor, category, len)?.finish())
    }

    pub fn try_from_slice(
        governor: &MemoryGovernor,
        category: MemoryCategory,
        source: &[u8],
    ) -> MemoryResult<Self> {
        let mut builder = TrackedBytesBuilder::try_zeroed(governor, category, source.len())?;
        builder.as_mut_slice().copy_from_slice(source);
        Ok(builder.finish())
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.bytes.capacity()
    }

    pub fn into_builder(self) -> TrackedBytesBuilder {
        TrackedBytesBuilder {
            bytes: self.bytes,
            lease: self.lease,
        }
    }
}

impl AsRef<[u8]> for TrackedBytes {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

/// Mutable only while building, with no API permitting unaccounted reallocation.
#[derive(Debug)]
pub struct TrackedBytesBuilder {
    bytes: Vec<u8>,
    lease: ByteLease,
}

impl TrackedBytesBuilder {
    pub fn try_zeroed(
        governor: &MemoryGovernor,
        category: MemoryCategory,
        len: usize,
    ) -> MemoryResult<Self> {
        let lease =
            governor.try_reserve_one(Cost::new(category, MemoryRepresentation::Cpu, len))?;
        Self::from_lease(lease, len)
    }

    /// Uses a previously admitted CPU or transient lease for a multi-resource plan.
    /// Resizes its accounting to `len` before allocating, then to real capacity.
    pub fn from_lease(mut lease: ByteLease, len: usize) -> MemoryResult<Self> {
        if lease.cost.representation == MemoryRepresentation::Gpu {
            return Err(MemoryError::InvalidByteRepresentation);
        }
        lease.try_resize(len)?;
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(len)?;
        // On failure the vector is freed before the earlier-created lease.
        lease.try_resize(bytes.capacity())?;
        bytes.resize(len, 0);
        Ok(Self { bytes, lease })
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    pub fn capacity(&self) -> usize {
        self.bytes.capacity()
    }

    pub fn finish(self) -> TrackedBytes {
        TrackedBytes {
            bytes: self.bytes,
            lease: self.lease,
        }
    }
}
