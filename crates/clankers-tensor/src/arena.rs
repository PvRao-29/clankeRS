//! Allocation manager for the scratch tensors an inference engine materialises
//! while adapting inputs and producing outputs.
//!
//! Every conversion (a borrowed input the backend cannot read directly) and every
//! produced output is a [`Tensor`] that has to come from somewhere. Routing those
//! allocations through a [`TensorArena`] gives the engine a single place to (a)
//! count how much heap traffic a run costs — the raw material for the copy /
//! allocation accounting the engine reports — and (b) eventually reuse buffers
//! across runs for the zero-allocation hot loop.

use crate::{DType, Shape, Tensor};

/// How the engine sources memory for converted inputs and produced outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AllocationPolicy {
    /// Allocate a fresh buffer for every conversion and every output. Simple and
    /// always correct; the default for v1.
    #[default]
    Dynamic,
    /// Reuse conversion buffers sized on the first run and kept for the engine's
    /// lifetime. A steady-state loop whose tensor shapes don't change performs no
    /// heap allocation for input conversions. Buffers are reused per input slot
    /// and are dropped/reallocated if a slot's dtype or shape changes.
    Preallocate,
}

/// Owns and accounts for the scratch tensors an inference engine allocates.
///
/// The arena keeps cumulative counters; the engine snapshots them before and
/// after a run to derive per-run allocation stats. Under
/// [`AllocationPolicy::Preallocate`] a steady-state loop will reuse buffers and
/// those per-run deltas fall to zero.
#[derive(Debug)]
pub struct TensorArena {
    policy: AllocationPolicy,
    allocations: usize,
    bytes_allocated: usize,
    /// Per-slot reusable buffers, populated under [`AllocationPolicy::Preallocate`]
    /// as tensors are checked back in. Empty under [`AllocationPolicy::Dynamic`].
    pool: Vec<Option<Tensor>>,
}

impl TensorArena {
    /// A new arena using `policy`.
    pub fn new(policy: AllocationPolicy) -> Self {
        TensorArena {
            policy,
            allocations: 0,
            bytes_allocated: 0,
            pool: Vec::new(),
        }
    }

    /// A dynamic-allocation arena (the default policy).
    pub fn dynamic() -> Self {
        TensorArena::new(AllocationPolicy::Dynamic)
    }

    /// The policy this arena was built with.
    pub fn policy(&self) -> AllocationPolicy {
        self.policy
    }

    /// Allocate a zero-initialised tensor of `dtype` and `shape`, recording the
    /// allocation against the cumulative counters.
    pub fn alloc(&mut self, dtype: DType, shape: Shape) -> Tensor {
        let tensor = Tensor::zeros(dtype, shape);
        self.allocations += 1;
        self.bytes_allocated += tensor.num_bytes();
        tensor
    }

    /// Check out a scratch tensor for input `slot`.
    ///
    /// Under [`AllocationPolicy::Preallocate`], if a previously checked-in buffer
    /// for this slot matches `dtype` and `shape`, it is reused with **no
    /// allocation**; otherwise a fresh tensor is allocated (and counted). Under
    /// [`AllocationPolicy::Dynamic`] this always allocates. Pair with
    /// [`checkin`](Self::checkin) to return the buffer for reuse.
    pub fn checkout(&mut self, slot: usize, dtype: DType, shape: Shape) -> Tensor {
        if self.policy == AllocationPolicy::Preallocate {
            if let Some(slot_buf) = self.pool.get_mut(slot) {
                if let Some(existing) = slot_buf.take() {
                    if existing.dtype() == dtype && existing.shape() == &shape {
                        return existing; // reuse — no allocation
                    }
                    // Shape/dtype changed: drop the stale buffer and reallocate.
                }
            }
        }
        self.alloc(dtype, shape)
    }

    /// Return a checked-out tensor to the pool so a future
    /// [`checkout`](Self::checkout) of the same `slot` can reuse it.
    ///
    /// A no-op (the tensor is dropped) under [`AllocationPolicy::Dynamic`].
    pub fn checkin(&mut self, slot: usize, tensor: Tensor) {
        if self.policy != AllocationPolicy::Preallocate {
            return;
        }
        if slot >= self.pool.len() {
            self.pool.resize_with(slot + 1, || None);
        }
        self.pool[slot] = Some(tensor);
    }

    /// Total number of allocations made over this arena's lifetime.
    pub fn total_allocations(&self) -> usize {
        self.allocations
    }

    /// Total bytes allocated over this arena's lifetime.
    pub fn total_bytes_allocated(&self) -> usize {
        self.bytes_allocated
    }
}

impl Default for TensorArena {
    fn default() -> Self {
        TensorArena::dynamic()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_counts_allocations_and_bytes() {
        let mut arena = TensorArena::dynamic();
        assert_eq!(arena.total_allocations(), 0);

        let a = arena.alloc(DType::F32, Shape::from([1, 4]));
        assert_eq!(a.num_bytes(), 16);
        assert_eq!(arena.total_allocations(), 1);
        assert_eq!(arena.total_bytes_allocated(), 16);

        let _b = arena.alloc(DType::U8, Shape::from([480, 640, 3]));
        assert_eq!(arena.total_allocations(), 2);
        assert_eq!(arena.total_bytes_allocated(), 16 + 480 * 640 * 3);
    }

    #[test]
    fn zeroed_by_default() {
        let mut arena = TensorArena::dynamic();
        let t = arena.alloc(DType::F32, Shape::from([8]));
        assert!(t.as_f32().unwrap().iter().all(|&x| x == 0.0));
    }

    #[test]
    fn policy_is_reported() {
        assert_eq!(TensorArena::dynamic().policy(), AllocationPolicy::Dynamic);
        assert_eq!(
            TensorArena::new(AllocationPolicy::Preallocate).policy(),
            AllocationPolicy::Preallocate
        );
    }

    #[test]
    fn preallocate_reuses_checked_in_buffers() {
        let mut arena = TensorArena::new(AllocationPolicy::Preallocate);
        let t = arena.checkout(0, DType::F32, Shape::from([1, 4]));
        assert_eq!(arena.total_allocations(), 1); // first checkout allocates
        arena.checkin(0, t);

        // Same slot, same shape → reused, no new allocation.
        let t2 = arena.checkout(0, DType::F32, Shape::from([1, 4]));
        assert_eq!(arena.total_allocations(), 1);
        arena.checkin(0, t2);

        // A different shape for the slot forces a reallocation.
        let _t3 = arena.checkout(0, DType::F32, Shape::from([1, 8]));
        assert_eq!(arena.total_allocations(), 2);
    }

    #[test]
    fn dynamic_never_reuses() {
        let mut arena = TensorArena::dynamic();
        let t = arena.checkout(0, DType::F32, Shape::from([1, 4]));
        arena.checkin(0, t); // dropped under Dynamic
        let _t2 = arena.checkout(0, DType::F32, Shape::from([1, 4]));
        assert_eq!(arena.total_allocations(), 2);
    }
}
