/// Determines how the MCTS process should be parallelized.
/// No functions; simply a marker trait.
pub enum ParallelizationPolicy {
	/// Executes multiple rollouts for a single node.
	SingeNodeMultipleRollouts { num_rollouts: u8 },

	/// Executes a single rollout for several nodes.
	MultipleNodesSingleRollout { num_nodes: u8 },
}

impl Default for ParallelizationPolicy {
	fn default() -> Self {
		Self::SingeNodeMultipleRollouts { num_rollouts: 4 }
	}
}
