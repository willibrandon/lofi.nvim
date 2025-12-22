//! Flow Matching schedulers for ACE-Step.
//!
//! Implements the FlowMatchEulerDiscreteScheduler, FlowMatchHeunDiscreteScheduler,
//! and FlowMatchPingPongScheduler from the ACE-Step codebase.
//! These are NOT Karras diffusion schedulers - they use flow matching formulation.

use ndarray::{Array4, Dimension};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, StandardNormal};

/// Scheduler type for diffusion process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SchedulerType {
    /// Euler ODE solver - fast, deterministic.
    #[default]
    Euler,
    /// Heun ODE solver - 2x slower, more accurate.
    Heun,
    /// PingPong SDE solver - stochastic, best quality.
    PingPong,
}

impl SchedulerType {
    /// Parses a scheduler type from a string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "euler" => Some(SchedulerType::Euler),
            "heun" => Some(SchedulerType::Heun),
            "pingpong" | "ping_pong" | "ping-pong" => Some(SchedulerType::PingPong),
            _ => None,
        }
    }

    /// Returns the string name of this scheduler.
    pub fn as_str(&self) -> &'static str {
        match self {
            SchedulerType::Euler => "euler",
            SchedulerType::Heun => "heun",
            SchedulerType::PingPong => "pingpong",
        }
    }
}

/// Common scheduler trait for flow matching diffusion.
pub trait Scheduler {
    /// Returns the current timestep value (sigma * 1000).
    fn timestep(&self) -> f32;

    /// Returns the current sigma (noise level, 0.0 to ~1.0).
    fn sigma(&self) -> f32;

    /// Performs one scheduler step to update the latent.
    fn step(&mut self, latent: &Array4<f32>, model_output: &Array4<f32>) -> Array4<f32>;

    /// Returns whether the scheduler has completed all steps.
    fn is_done(&self) -> bool;

    /// Returns the current step index.
    fn current_step(&self) -> usize;

    /// Returns the total number of steps.
    fn num_steps(&self) -> u32;

    /// Resets the scheduler to the initial state.
    fn reset(&mut self);

    /// Returns all sigmas for the schedule.
    fn sigmas(&self) -> &[f32];

    /// Returns all timesteps for the schedule.
    fn timesteps(&self) -> &[f32];

    /// Returns true if this scheduler requires two model evaluations per user step.
    fn requires_two_evaluations(&self) -> bool {
        false
    }

    /// Returns the user-visible step (for schedulers that use internal sub-steps).
    fn user_step(&self) -> usize {
        self.current_step()
    }

    /// Returns the total user-visible steps.
    fn user_num_steps(&self) -> u32 {
        self.num_steps()
    }
}

/// Flow Matching Euler scheduler.
///
/// Based on FlowMatchEulerDiscreteScheduler from ACE-Step.
/// Uses shifted sigmas: `shift * sigma / (1 + (shift - 1) * sigma)`
#[derive(Debug, Clone)]
pub struct EulerScheduler {
    /// Total number of inference steps.
    num_steps: u32,
    /// Omega scale for mean shifting (default 10.0).
    omega: f32,
    /// Sigma values for each timestep (from ~1.0 to 0.0).
    sigmas: Vec<f32>,
    /// Timesteps for each step (sigmas * 1000).
    timesteps: Vec<f32>,
    /// Current step index.
    current_step: usize,
}

impl EulerScheduler {
    /// Creates a new Flow Matching Euler scheduler.
    ///
    /// # Arguments
    ///
    /// * `num_steps` - Number of diffusion steps (typically 60)
    /// * `shift` - Shift parameter (default 3.0)
    /// * `omega` - Omega scale for mean shifting (default 10.0)
    pub fn new(num_steps: u32, shift: f32, omega: f32) -> Self {
        let (sigmas, timesteps) = compute_flow_matching_schedule(num_steps, shift);

        Self {
            num_steps,
            omega,
            sigmas,
            timesteps,
            current_step: 0,
        }
    }

    /// Creates a scheduler with default ACE-Step parameters.
    pub fn default_ace_step(num_steps: u32) -> Self {
        Self::new(num_steps, 3.0, 10.0)
    }

    /// Returns the next sigma (noise level for next step).
    pub fn next_sigma(&self) -> f32 {
        self.sigmas[self.current_step + 1]
    }
}

impl Scheduler for EulerScheduler {
    fn timestep(&self) -> f32 {
        self.timesteps[self.current_step]
    }

    fn sigma(&self) -> f32 {
        self.sigmas[self.current_step]
    }

    fn step(&mut self, latent: &Array4<f32>, model_output: &Array4<f32>) -> Array4<f32> {
        let sigma = self.sigma();
        let sigma_next = self.next_sigma();
        let dt = sigma_next - sigma; // This is negative (going from high sigma to low)

        // Compute dx = dt * model_output
        let dx = model_output.mapv(|v| v * dt);

        // Apply omega mean shifting for stability
        let omega_scaled = logistic(self.omega, 0.9, 1.1, 0.0, 0.1);
        let mean = dx.mean().unwrap_or(0.0);
        let dx_shifted = dx.mapv(|v| (v - mean) * omega_scaled + mean);

        // Update latent: x_next = x + dx_shifted
        let next_latent = latent + &dx_shifted;

        // Advance to next step
        self.current_step += 1;

        next_latent
    }

    fn is_done(&self) -> bool {
        self.current_step >= self.num_steps as usize
    }

    fn current_step(&self) -> usize {
        self.current_step
    }

    fn num_steps(&self) -> u32 {
        self.num_steps
    }

    fn reset(&mut self) {
        self.current_step = 0;
    }

    fn sigmas(&self) -> &[f32] {
        &self.sigmas
    }

    fn timesteps(&self) -> &[f32] {
        &self.timesteps
    }
}

// ============================================================================
// HeunScheduler - 2nd order Heun's method (predictor-corrector)
// ============================================================================

/// Flow Matching Heun scheduler.
///
/// Based on FlowMatchHeunDiscreteScheduler from ACE-Step.
/// Uses Heun's method (2nd order Runge-Kutta) for more accurate integration.
/// Requires 2 model evaluations per user-visible step.
#[derive(Debug, Clone)]
pub struct HeunScheduler {
    /// Total number of user-visible inference steps.
    num_steps: u32,
    /// Omega scale for mean shifting (default 10.0).
    omega: f32,
    /// Sigma values for each internal timestep (interleaved for Heun).
    sigmas: Vec<f32>,
    /// Timesteps for each internal step.
    timesteps: Vec<f32>,
    /// Current internal step index (0 to 2*num_steps-1).
    current_step: usize,
    /// Stored derivative from first-order prediction.
    prev_derivative: Option<Array4<f32>>,
    /// Stored delta-t from first-order prediction.
    dt: Option<f32>,
    /// Stored sample from first-order prediction.
    prev_sample: Option<Array4<f32>>,
}

impl HeunScheduler {
    /// Creates a new Flow Matching Heun scheduler.
    pub fn new(num_steps: u32, shift: f32, omega: f32) -> Self {
        let (base_sigmas, _) = compute_flow_matching_schedule(num_steps, shift);

        // Heun scheduler needs interleaved sigmas and timesteps
        // timesteps[1:].repeat_interleave(2) with timesteps[:1] prepended
        // sigmas: sigmas[:1], sigmas[1:-1].repeat_interleave(2), sigmas[-1:]
        let num_train_timesteps = 1000.0_f32;

        // Build interleaved timesteps for Heun
        let mut timesteps = Vec::with_capacity(2 * num_steps as usize - 1);
        timesteps.push(base_sigmas[0] * num_train_timesteps);
        for i in 1..num_steps as usize {
            let t = base_sigmas[i] * num_train_timesteps;
            timesteps.push(t);
            timesteps.push(t);
        }

        // Build interleaved sigmas for Heun
        let mut sigmas = Vec::with_capacity(2 * num_steps as usize);
        sigmas.push(base_sigmas[0]);
        for i in 1..base_sigmas.len() - 1 {
            sigmas.push(base_sigmas[i]);
            sigmas.push(base_sigmas[i]);
        }
        sigmas.push(0.0); // Final sigma

        Self {
            num_steps,
            omega,
            sigmas,
            timesteps,
            current_step: 0,
            prev_derivative: None,
            dt: None,
            prev_sample: None,
        }
    }

    /// Creates a scheduler with default ACE-Step parameters.
    pub fn default_ace_step(num_steps: u32) -> Self {
        Self::new(num_steps, 3.0, 10.0)
    }

    /// Returns true if in first-order (prediction) state.
    fn state_in_first_order(&self) -> bool {
        self.dt.is_none()
    }
}

impl Scheduler for HeunScheduler {
    fn timestep(&self) -> f32 {
        self.timesteps[self.current_step.min(self.timesteps.len() - 1)]
    }

    fn sigma(&self) -> f32 {
        self.sigmas[self.current_step]
    }

    fn step(&mut self, latent: &Array4<f32>, model_output: &Array4<f32>) -> Array4<f32> {
        let omega_scaled = logistic(self.omega, 0.9, 1.1, 0.0, 0.1);

        if self.state_in_first_order() {
            // First order: prediction step
            let sigma = self.sigmas[self.current_step];
            let sigma_next = self.sigmas[self.current_step + 1];
            let sigma_hat = sigma;

            // 1. Compute denoised prediction
            let denoised = latent - &model_output.mapv(|v| v * sigma);

            // 2. Compute derivative
            let derivative = (latent - &denoised).mapv(|v| v / sigma_hat);

            // 3. Delta timestep
            let dt = sigma_next - sigma_hat;

            // Store for 2nd order step
            self.prev_derivative = Some(derivative);
            self.dt = Some(dt);
            self.prev_sample = Some(latent.clone());

            // Advance step
            self.current_step += 1;

            // For first order, return predicted next sample for model evaluation
            let dx = self.prev_derivative.as_ref().unwrap().mapv(|v| v * dt);
            let mean = dx.mean().unwrap_or(0.0);
            let dx_shifted = dx.mapv(|v| (v - mean) * omega_scaled + mean);
            latent + &dx_shifted
        } else {
            // Second order: correction step
            let sigma_next = self.sigmas[self.current_step];

            // 1. Compute denoised prediction at predicted point
            let denoised = latent - &model_output.mapv(|v| v * sigma_next);

            // 2. Compute new derivative
            let derivative = if sigma_next > 0.0 {
                (latent - &denoised).mapv(|v| v / sigma_next)
            } else {
                Array4::zeros(latent.raw_dim())
            };

            // 3. Average with previous derivative (Heun's method)
            let prev_deriv = self.prev_derivative.take().unwrap();
            let avg_derivative = (&prev_deriv + &derivative).mapv(|v| v * 0.5);

            // 4. Get stored values
            let dt = self.dt.take().unwrap();
            let sample = self.prev_sample.take().unwrap();

            // 5. Apply update with omega mean shifting
            let dx = avg_derivative.mapv(|v| v * dt);
            let mean = dx.mean().unwrap_or(0.0);
            let dx_shifted = dx.mapv(|v| (v - mean) * omega_scaled + mean);
            let prev_sample = &sample + &dx_shifted;

            // Advance step
            self.current_step += 1;

            prev_sample
        }
    }

    fn is_done(&self) -> bool {
        self.current_step >= self.sigmas.len() - 1
    }

    fn current_step(&self) -> usize {
        self.current_step
    }

    fn num_steps(&self) -> u32 {
        // Internal steps are doubled for Heun
        (self.sigmas.len() - 1) as u32
    }

    fn reset(&mut self) {
        self.current_step = 0;
        self.prev_derivative = None;
        self.dt = None;
        self.prev_sample = None;
    }

    fn sigmas(&self) -> &[f32] {
        &self.sigmas
    }

    fn timesteps(&self) -> &[f32] {
        &self.timesteps
    }

    fn requires_two_evaluations(&self) -> bool {
        true
    }

    fn user_step(&self) -> usize {
        self.current_step / 2
    }

    fn user_num_steps(&self) -> u32 {
        self.num_steps
    }
}

// ============================================================================
// PingPongScheduler - Stochastic SDE solver
// ============================================================================

/// Flow Matching PingPong scheduler.
///
/// Based on FlowMatchPingPongScheduler from ACE-Step.
/// Uses stochastic sampling - adds fresh noise at each step for exploration.
/// Generally produces highest quality but less reproducible results.
#[derive(Debug, Clone)]
pub struct PingPongScheduler {
    /// Total number of inference steps.
    num_steps: u32,
    /// Omega scale for mean shifting (reserved for future use).
    #[allow(dead_code)]
    omega: f32,
    /// Sigma values for each timestep (from ~1.0 to 0.0).
    sigmas: Vec<f32>,
    /// Timesteps for each step (sigmas * 1000).
    timesteps: Vec<f32>,
    /// Current step index.
    current_step: usize,
    /// Random number generator for stochastic noise.
    rng: ChaCha8Rng,
}

impl PingPongScheduler {
    /// Creates a new Flow Matching PingPong scheduler.
    pub fn new(num_steps: u32, shift: f32, omega: f32, seed: u64) -> Self {
        let (sigmas, timesteps) = compute_flow_matching_schedule(num_steps, shift);

        Self {
            num_steps,
            omega,
            sigmas,
            timesteps,
            current_step: 0,
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    /// Creates a scheduler with default ACE-Step parameters.
    pub fn default_ace_step(num_steps: u32, seed: u64) -> Self {
        Self::new(num_steps, 3.0, 10.0, seed)
    }

    /// Returns the next sigma (noise level for next step).
    fn next_sigma(&self) -> f32 {
        self.sigmas[self.current_step + 1]
    }
}

impl Scheduler for PingPongScheduler {
    fn timestep(&self) -> f32 {
        self.timesteps[self.current_step]
    }

    fn sigma(&self) -> f32 {
        self.sigmas[self.current_step]
    }

    fn step(&mut self, latent: &Array4<f32>, model_output: &Array4<f32>) -> Array4<f32> {
        let sigma = self.sigma();
        let sigma_next = self.next_sigma();

        // PingPong step (SDE formulation):
        // 1. Compute denoised sample: denoised = sample - sigma * model_output
        let denoised = latent - &model_output.mapv(|v| v * sigma);

        // 2. Generate fresh noise for stochastic exploration
        let noise = generate_noise_like(latent, &mut self.rng);

        // 3. Mix denoised with fresh noise: prev_sample = (1 - sigma_next) * denoised + sigma_next * noise
        let one_minus_sigma_next = 1.0 - sigma_next;
        let prev_sample = denoised.mapv(|v| v * one_minus_sigma_next)
            + noise.mapv(|v| v * sigma_next);

        // Advance to next step
        self.current_step += 1;

        prev_sample
    }

    fn is_done(&self) -> bool {
        self.current_step >= self.num_steps as usize
    }

    fn current_step(&self) -> usize {
        self.current_step
    }

    fn num_steps(&self) -> u32 {
        self.num_steps
    }

    fn reset(&mut self) {
        self.current_step = 0;
    }

    fn sigmas(&self) -> &[f32] {
        &self.sigmas
    }

    fn timesteps(&self) -> &[f32] {
        &self.timesteps
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Computes the flow matching sigma schedule with shifting.
///
/// Returns (sigmas, timesteps) where sigmas has num_steps + 1 elements (final is 0.0).
fn compute_flow_matching_schedule(num_steps: u32, shift: f32) -> (Vec<f32>, Vec<f32>) {
    let num_train_timesteps = 1000.0_f32;
    let sigma_max = 1.0_f32;

    // Linear interpolation from max to small positive value with shift applied
    // Use num_steps as denominator so last sigma is small but non-zero
    // (prevents division by zero in Heun scheduler)
    let mut sigmas: Vec<f32> = (0..num_steps)
        .map(|i| {
            // t goes from 1.0 to small positive (not 0)
            let t = sigma_max - (i as f32 / num_steps as f32) * sigma_max;
            // Apply shift: shift * t / (1 + (shift - 1) * t)
            shift * t / (1.0 + (shift - 1.0) * t)
        })
        .collect();

    // Append final sigma of 0 (only used as terminal condition)
    sigmas.push(0.0);

    // Timesteps are sigmas * num_train_timesteps
    let timesteps: Vec<f32> = sigmas
        .iter()
        .take(num_steps as usize)
        .map(|s| s * num_train_timesteps)
        .collect();

    (sigmas, timesteps)
}

/// Logistic function for omega scaling.
/// Maps input x to range [lower, upper] with sigmoid shape.
fn logistic(x: f32, lower: f32, upper: f32, x0: f32, k: f32) -> f32 {
    lower + (upper - lower) / (1.0 + (-k * (x - x0)).exp())
}

/// Generates random noise with the same shape as the input array.
fn generate_noise_like(arr: &Array4<f32>, rng: &mut ChaCha8Rng) -> Array4<f32> {
    let shape = arr.raw_dim();
    let size = shape.size();
    let noise: Vec<f32> = (0..size)
        .map(|_| StandardNormal.sample(rng))
        .collect();

    Array4::from_shape_vec(shape, noise).unwrap()
}

// ============================================================================
// Scheduler enum for dynamic dispatch
// ============================================================================

/// Dynamic scheduler wrapper that can hold any scheduler type.
pub enum DynScheduler {
    Euler(EulerScheduler),
    Heun(HeunScheduler),
    PingPong(PingPongScheduler),
}

impl DynScheduler {
    /// Returns the current timestep value (sigma * 1000).
    pub fn timestep(&self) -> f32 {
        match self {
            DynScheduler::Euler(s) => s.timestep(),
            DynScheduler::Heun(s) => s.timestep(),
            DynScheduler::PingPong(s) => s.timestep(),
        }
    }

    /// Returns the current sigma (noise level).
    pub fn sigma(&self) -> f32 {
        match self {
            DynScheduler::Euler(s) => s.sigma(),
            DynScheduler::Heun(s) => s.sigma(),
            DynScheduler::PingPong(s) => s.sigma(),
        }
    }

    /// Performs one scheduler step.
    pub fn step(&mut self, latent: &Array4<f32>, model_output: &Array4<f32>) -> Array4<f32> {
        match self {
            DynScheduler::Euler(s) => s.step(latent, model_output),
            DynScheduler::Heun(s) => s.step(latent, model_output),
            DynScheduler::PingPong(s) => s.step(latent, model_output),
        }
    }

    /// Returns whether the scheduler has completed all steps.
    pub fn is_done(&self) -> bool {
        match self {
            DynScheduler::Euler(s) => s.is_done(),
            DynScheduler::Heun(s) => s.is_done(),
            DynScheduler::PingPong(s) => s.is_done(),
        }
    }

    /// Returns the current step index.
    pub fn current_step(&self) -> usize {
        match self {
            DynScheduler::Euler(s) => s.current_step(),
            DynScheduler::Heun(s) => s.current_step(),
            DynScheduler::PingPong(s) => s.current_step(),
        }
    }

    /// Returns the total number of internal steps.
    pub fn num_steps(&self) -> u32 {
        match self {
            DynScheduler::Euler(s) => s.num_steps(),
            DynScheduler::Heun(s) => s.num_steps(),
            DynScheduler::PingPong(s) => s.num_steps(),
        }
    }

    /// Resets the scheduler.
    pub fn reset(&mut self) {
        match self {
            DynScheduler::Euler(s) => s.reset(),
            DynScheduler::Heun(s) => s.reset(),
            DynScheduler::PingPong(s) => s.reset(),
        }
    }

    /// Returns all sigmas.
    pub fn sigmas(&self) -> &[f32] {
        match self {
            DynScheduler::Euler(s) => s.sigmas(),
            DynScheduler::Heun(s) => s.sigmas(),
            DynScheduler::PingPong(s) => s.sigmas(),
        }
    }

    /// Returns all timesteps.
    pub fn timesteps(&self) -> &[f32] {
        match self {
            DynScheduler::Euler(s) => s.timesteps(),
            DynScheduler::Heun(s) => s.timesteps(),
            DynScheduler::PingPong(s) => s.timesteps(),
        }
    }

    /// Returns true if this scheduler requires two model evaluations per user step.
    pub fn requires_two_evaluations(&self) -> bool {
        match self {
            DynScheduler::Euler(s) => s.requires_two_evaluations(),
            DynScheduler::Heun(s) => s.requires_two_evaluations(),
            DynScheduler::PingPong(s) => s.requires_two_evaluations(),
        }
    }

    /// Returns the user-visible step.
    pub fn user_step(&self) -> usize {
        match self {
            DynScheduler::Euler(s) => s.user_step(),
            DynScheduler::Heun(s) => s.user_step(),
            DynScheduler::PingPong(s) => s.user_step(),
        }
    }

    /// Returns the total user-visible steps.
    pub fn user_num_steps(&self) -> u32 {
        match self {
            DynScheduler::Euler(s) => s.user_num_steps(),
            DynScheduler::Heun(s) => s.user_num_steps(),
            DynScheduler::PingPong(s) => s.user_num_steps(),
        }
    }
}

/// Creates a scheduler of the specified type.
///
/// # Arguments
/// * `scheduler_type` - The type of scheduler to create
/// * `num_steps` - Number of inference steps
/// * `seed` - Random seed (only used for PingPong scheduler)
pub fn create_scheduler(scheduler_type: SchedulerType, num_steps: u32, seed: u64) -> DynScheduler {
    match scheduler_type {
        SchedulerType::Euler => DynScheduler::Euler(EulerScheduler::default_ace_step(num_steps)),
        SchedulerType::Heun => DynScheduler::Heun(HeunScheduler::default_ace_step(num_steps)),
        SchedulerType::PingPong => DynScheduler::PingPong(PingPongScheduler::default_ace_step(num_steps, seed)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduler_type_parsing() {
        assert_eq!(SchedulerType::parse("euler"), Some(SchedulerType::Euler));
        assert_eq!(SchedulerType::parse("heun"), Some(SchedulerType::Heun));
        assert_eq!(SchedulerType::parse("pingpong"), Some(SchedulerType::PingPong));
        assert_eq!(SchedulerType::parse("ping_pong"), Some(SchedulerType::PingPong));
        assert_eq!(SchedulerType::parse("ping-pong"), Some(SchedulerType::PingPong));
        assert_eq!(SchedulerType::parse("invalid"), None);
    }

    #[test]
    fn scheduler_type_as_str() {
        assert_eq!(SchedulerType::Euler.as_str(), "euler");
        assert_eq!(SchedulerType::Heun.as_str(), "heun");
        assert_eq!(SchedulerType::PingPong.as_str(), "pingpong");
    }

    // ========== Euler Scheduler Tests ==========

    #[test]
    fn euler_scheduler_creation() {
        let scheduler = EulerScheduler::default_ace_step(60);
        assert_eq!(scheduler.num_steps(), 60);
        assert_eq!(scheduler.current_step(), 0);
        assert!(!scheduler.is_done());
    }

    #[test]
    fn euler_scheduler_sigmas() {
        let scheduler = EulerScheduler::default_ace_step(60);
        let sigmas = scheduler.sigmas();

        // Should have num_steps + 1 sigmas (including final 0)
        assert_eq!(sigmas.len(), 61);

        // First sigma should be ~1.0 (shift*1/(1+(shift-1)*1) = 3/3 = 1.0)
        assert!((sigmas[0] - 1.0).abs() < 0.01, "First sigma should be ~1.0, got {}", sigmas[0]);

        // Last sigma should be 0.0
        assert_eq!(sigmas[sigmas.len() - 1], 0.0);

        // Sigmas should be monotonically decreasing
        for i in 1..sigmas.len() {
            assert!(sigmas[i] <= sigmas[i - 1], "Sigma {} ({}) > sigma {} ({})", i, sigmas[i], i - 1, sigmas[i - 1]);
        }
    }

    #[test]
    fn euler_scheduler_timesteps() {
        let scheduler = EulerScheduler::default_ace_step(60);
        let timesteps = scheduler.timesteps();

        // First timestep should be ~1000 (sigma ~1.0 * 1000)
        assert!(timesteps[0] > 900.0, "First timestep should be ~1000, got {}", timesteps[0]);
    }

    #[test]
    fn euler_scheduler_step() {
        let mut scheduler = EulerScheduler::default_ace_step(60);

        let latent = Array4::zeros((1, 8, 16, 100));
        let noise_pred = Array4::ones((1, 8, 16, 100));

        let initial_step = scheduler.current_step();
        let _ = scheduler.step(&latent, &noise_pred);

        assert_eq!(scheduler.current_step(), initial_step + 1);
    }

    #[test]
    fn euler_scheduler_completes() {
        let mut scheduler = EulerScheduler::default_ace_step(10);
        let latent = Array4::zeros((1, 8, 16, 100));
        let noise_pred = Array4::ones((1, 8, 16, 100));

        for _ in 0..10 {
            assert!(!scheduler.is_done());
            let _ = scheduler.step(&latent, &noise_pred);
        }
        assert!(scheduler.is_done());
    }

    // ========== Heun Scheduler Tests ==========

    #[test]
    fn heun_scheduler_creation() {
        let scheduler = HeunScheduler::default_ace_step(60);
        // Heun has internal steps doubled
        assert_eq!(scheduler.user_num_steps(), 60);
        assert_eq!(scheduler.current_step(), 0);
        assert!(!scheduler.is_done());
    }

    #[test]
    fn heun_scheduler_requires_two_evaluations() {
        let scheduler = HeunScheduler::default_ace_step(60);
        assert!(scheduler.requires_two_evaluations());
    }

    #[test]
    fn heun_scheduler_step() {
        let mut scheduler = HeunScheduler::default_ace_step(10);

        let latent = Array4::zeros((1, 8, 16, 100));
        let noise_pred = Array4::ones((1, 8, 16, 100));

        // First call: prediction step
        assert!(scheduler.state_in_first_order());
        let mid_latent = scheduler.step(&latent, &noise_pred);

        // Second call: correction step
        assert!(!scheduler.state_in_first_order());
        let _ = scheduler.step(&mid_latent, &noise_pred);

        // Back to first order
        assert!(scheduler.state_in_first_order());
    }

    #[test]
    fn heun_scheduler_user_step() {
        let mut scheduler = HeunScheduler::default_ace_step(10);
        let latent = Array4::zeros((1, 8, 16, 100));
        let noise_pred = Array4::ones((1, 8, 16, 100));

        assert_eq!(scheduler.user_step(), 0);

        // Two internal steps = one user step
        let mid = scheduler.step(&latent, &noise_pred);
        assert_eq!(scheduler.user_step(), 0);
        scheduler.step(&mid, &noise_pred);
        assert_eq!(scheduler.user_step(), 1);
    }

    // ========== PingPong Scheduler Tests ==========

    #[test]
    fn pingpong_scheduler_creation() {
        let scheduler = PingPongScheduler::default_ace_step(60, 42);
        assert_eq!(scheduler.num_steps(), 60);
        assert_eq!(scheduler.current_step(), 0);
        assert!(!scheduler.is_done());
    }

    #[test]
    fn pingpong_scheduler_step() {
        let mut scheduler = PingPongScheduler::default_ace_step(60, 42);

        let latent = Array4::zeros((1, 8, 16, 100));
        let noise_pred = Array4::ones((1, 8, 16, 100));

        let initial_step = scheduler.current_step();
        let _ = scheduler.step(&latent, &noise_pred);

        assert_eq!(scheduler.current_step(), initial_step + 1);
    }

    #[test]
    fn pingpong_scheduler_stochastic() {
        // Run same scheduler twice with same seed - should produce same results
        let mut scheduler1 = PingPongScheduler::default_ace_step(10, 42);
        let mut scheduler2 = PingPongScheduler::default_ace_step(10, 42);

        let latent = Array4::ones((1, 8, 16, 50));
        let noise_pred = Array4::ones((1, 8, 16, 50));

        let result1 = scheduler1.step(&latent, &noise_pred);
        let result2 = scheduler2.step(&latent, &noise_pred);

        // Same seed should produce identical results
        assert_eq!(result1, result2);
    }

    #[test]
    fn pingpong_scheduler_different_seeds() {
        // Different seeds should produce different results
        let mut scheduler1 = PingPongScheduler::default_ace_step(10, 42);
        let mut scheduler2 = PingPongScheduler::default_ace_step(10, 123);

        let latent = Array4::ones((1, 8, 16, 50));
        let noise_pred = Array4::ones((1, 8, 16, 50));

        let result1 = scheduler1.step(&latent, &noise_pred);
        let result2 = scheduler2.step(&latent, &noise_pred);

        // Different seeds should produce different results
        assert_ne!(result1, result2);
    }

    // ========== create_scheduler Tests ==========

    #[test]
    fn create_scheduler_euler() {
        let scheduler = create_scheduler(SchedulerType::Euler, 60, 42);
        assert!(matches!(scheduler, DynScheduler::Euler(_)));
        assert_eq!(scheduler.num_steps(), 60);
    }

    #[test]
    fn create_scheduler_heun() {
        let scheduler = create_scheduler(SchedulerType::Heun, 60, 42);
        assert!(matches!(scheduler, DynScheduler::Heun(_)));
        assert!(scheduler.requires_two_evaluations());
    }

    #[test]
    fn create_scheduler_pingpong() {
        let scheduler = create_scheduler(SchedulerType::PingPong, 60, 42);
        assert!(matches!(scheduler, DynScheduler::PingPong(_)));
        assert_eq!(scheduler.num_steps(), 60);
    }

    // ========== Helper Function Tests ==========

    #[test]
    fn logistic_function() {
        // At x=0 with x0=0, should be at midpoint
        let mid = logistic(0.0, 0.9, 1.1, 0.0, 0.1);
        assert!((mid - 1.0).abs() < 0.01, "Logistic at x=0 should be ~1.0, got {}", mid);

        // At large positive x, should approach upper bound
        let high = logistic(100.0, 0.9, 1.1, 0.0, 0.1);
        assert!(high > 1.09, "Logistic at large x should be ~1.1, got {}", high);
    }

    #[test]
    fn compute_schedule_basic() {
        let (sigmas, timesteps) = compute_flow_matching_schedule(60, 3.0);

        assert_eq!(sigmas.len(), 61); // num_steps + 1
        assert_eq!(timesteps.len(), 60);

        // First sigma should be ~1.0
        assert!((sigmas[0] - 1.0).abs() < 0.01);

        // Last sigma should be 0.0
        assert_eq!(sigmas[60], 0.0);

        // Timesteps should be sigmas * 1000
        for i in 0..60 {
            assert!((timesteps[i] - sigmas[i] * 1000.0).abs() < 0.01);
        }
    }

    #[test]
    fn generate_noise_shape() {
        let arr = Array4::zeros((1, 8, 16, 100));
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let noise = generate_noise_like(&arr, &mut rng);

        assert_eq!(noise.shape(), arr.shape());
    }
}
