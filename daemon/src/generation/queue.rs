//! Generation queue for managing pending jobs.
//!
//! Implements a priority queue for generation jobs with a maximum capacity of 10.
//! High-priority jobs are inserted at the front of the queue.

use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::types::{GenerationJob, JobPriority};

/// Maximum number of jobs allowed in the queue.
pub const MAX_QUEUE_SIZE: usize = 10;

/// A priority queue for generation jobs.
///
/// The queue has a maximum capacity of 10 jobs. High-priority jobs
/// are inserted at the front, normal priority at the back.
#[derive(Debug)]
pub struct GenerationQueue {
    jobs: VecDeque<GenerationJob>,
}

impl Default for GenerationQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl GenerationQueue {
    /// Creates a new empty generation queue.
    pub fn new() -> Self {
        Self {
            jobs: VecDeque::with_capacity(MAX_QUEUE_SIZE),
        }
    }

    /// Adds a job to the queue with the given priority.
    ///
    /// High-priority jobs are inserted at the front of the queue,
    /// normal priority jobs at the back.
    ///
    /// Returns `Err` if the queue is full.
    pub fn add(&mut self, mut job: GenerationJob) -> Result<usize, QueueFullError> {
        if self.is_full() {
            return Err(QueueFullError {
                current_size: self.jobs.len(),
            });
        }

        let position = match job.priority {
            JobPriority::High => {
                // Insert at front, after any other high-priority jobs
                let insert_pos = self
                    .jobs
                    .iter()
                    .position(|j| j.priority != JobPriority::High)
                    .unwrap_or(self.jobs.len());
                job.set_queued(insert_pos as u8);
                self.jobs.insert(insert_pos, job);
                // Update positions for jobs after the insertion point
                self.update_positions();
                insert_pos
            }
            JobPriority::Normal => {
                let pos = self.jobs.len();
                job.set_queued(pos as u8);
                self.jobs.push_back(job);
                pos
            }
        };

        Ok(position)
    }

    /// Removes and returns the next job to process.
    ///
    /// Returns `None` if the queue is empty.
    pub fn pop_next(&mut self) -> Option<GenerationJob> {
        let job = self.jobs.pop_front();
        if job.is_some() {
            self.update_positions();
        }
        job
    }

    /// Returns the number of jobs in the queue.
    pub fn len(&self) -> usize {
        self.jobs.len()
    }

    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    /// Returns true if the queue is at maximum capacity (10 jobs).
    pub fn is_full(&self) -> bool {
        self.jobs.len() >= MAX_QUEUE_SIZE
    }

    /// Returns the position of a job in the queue by job_id.
    ///
    /// Returns `None` if the job is not found.
    pub fn get_position(&self, job_id: &str) -> Option<usize> {
        self.jobs.iter().position(|j| j.job_id == job_id)
    }

    /// Returns a reference to a job by job_id.
    pub fn get_job(&self, job_id: &str) -> Option<&GenerationJob> {
        self.jobs.iter().find(|j| j.job_id == job_id)
    }

    /// Returns a mutable reference to a job by job_id.
    pub fn get_job_mut(&mut self, job_id: &str) -> Option<&mut GenerationJob> {
        self.jobs.iter_mut().find(|j| j.job_id == job_id)
    }

    /// Updates queue positions for all jobs after modifications.
    fn update_positions(&mut self) {
        for (i, job) in self.jobs.iter_mut().enumerate() {
            job.queue_position = Some(i as u8);
        }
    }
}

/// Error returned when the queue is full.
#[derive(Debug, Clone)]
pub struct QueueFullError {
    /// Current number of jobs in the queue.
    pub current_size: usize,
}

impl std::fmt::Display for QueueFullError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Queue is full ({} jobs). Maximum capacity is {}.",
            self.current_size, MAX_QUEUE_SIZE
        )
    }
}

impl std::error::Error for QueueFullError {}

/// Message sent to the queue processor.
#[derive(Debug)]
pub enum QueueMessage {
    /// A new job has been added to the queue.
    JobAdded(Box<GenerationJob>),
    /// Request to shut down the processor.
    Shutdown,
}

/// Result of processing a job.
#[derive(Debug)]
pub enum JobResult {
    /// Job completed successfully with the path to the generated file.
    Complete {
        job_id: String,
        track_id: String,
        path: String,
        duration_sec: f32,
        generation_time_sec: f32,
    },
    /// Job failed with an error.
    Failed {
        job_id: String,
        track_id: String,
        error_code: String,
        error_message: String,
    },
}

/// A thread-safe queue processor that handles jobs in the background.
pub struct QueueProcessor {
    /// Channel to send jobs to the processor.
    sender: Sender<QueueMessage>,
    /// Handle to the processor thread.
    thread_handle: Option<JoinHandle<()>>,
    /// Shared queue state for position queries.
    queue: Arc<Mutex<GenerationQueue>>,
    /// Channel to receive job results.
    result_receiver: Receiver<JobResult>,
}

impl QueueProcessor {
    /// Creates a new queue processor.
    ///
    /// The processor starts a background thread that processes jobs serially.
    /// The `process_fn` is called for each job and should perform the actual generation.
    pub fn new<F>(process_fn: F) -> Self
    where
        F: Fn(GenerationJob) -> JobResult + Send + 'static,
    {
        let (job_sender, job_receiver) = mpsc::channel::<QueueMessage>();
        let (result_sender, result_receiver) = mpsc::channel::<JobResult>();
        let queue = Arc::new(Mutex::new(GenerationQueue::new()));
        let queue_clone = Arc::clone(&queue);

        let thread_handle = thread::spawn(move || {
            Self::processor_loop(job_receiver, result_sender, queue_clone, process_fn);
        });

        Self {
            sender: job_sender,
            thread_handle: Some(thread_handle),
            queue,
            result_receiver,
        }
    }

    /// Submits a job to the queue for processing.
    ///
    /// Returns the queue position if successful, or an error if the queue is full.
    pub fn submit(&self, job: GenerationJob) -> Result<usize, QueueFullError> {
        let mut queue = self.queue.lock().unwrap();
        let position = queue.add(job.clone())?;
        drop(queue);

        // Send to processor thread
        self.sender.send(QueueMessage::JobAdded(Box::new(job))).ok();

        Ok(position)
    }

    /// Returns the current queue length.
    pub fn queue_len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    /// Returns true if the queue is full.
    pub fn is_full(&self) -> bool {
        self.queue.lock().unwrap().is_full()
    }

    /// Returns the position of a job by job_id.
    pub fn get_position(&self, job_id: &str) -> Option<usize> {
        self.queue.lock().unwrap().get_position(job_id)
    }

    /// Tries to receive a job result without blocking.
    pub fn try_recv_result(&self) -> Option<JobResult> {
        self.result_receiver.try_recv().ok()
    }

    /// Shuts down the processor.
    pub fn shutdown(&mut self) {
        self.sender.send(QueueMessage::Shutdown).ok();
        if let Some(handle) = self.thread_handle.take() {
            handle.join().ok();
        }
    }

    /// The main processing loop running in the background thread.
    fn processor_loop<F>(
        receiver: Receiver<QueueMessage>,
        result_sender: Sender<JobResult>,
        queue: Arc<Mutex<GenerationQueue>>,
        process_fn: F,
    ) where
        F: Fn(GenerationJob) -> JobResult + Send + 'static,
    {
        loop {
            // Wait for a message
            match receiver.recv() {
                Ok(QueueMessage::JobAdded(_)) => {
                    // Pop the next job from the queue and process it
                    let job = {
                        let mut q = queue.lock().unwrap();
                        q.pop_next()
                    };

                    if let Some(mut job) = job {
                        job.set_generating();
                        let result = process_fn(job);
                        result_sender.send(result).ok();
                    }
                }
                Ok(QueueMessage::Shutdown) => {
                    break;
                }
                Err(_) => {
                    // Channel closed, exit
                    break;
                }
            }
        }
    }
}

impl Drop for QueueProcessor {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::JobStatus;

    fn create_test_job(priority: JobPriority) -> GenerationJob {
        GenerationJob::new(
            "test prompt".to_string(),
            30,
            Some(42),
            priority,
            "v1",
        )
    }

    #[test]
    fn queue_new_is_empty() {
        let queue = GenerationQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        assert!(!queue.is_full());
    }

    #[test]
    fn queue_add_normal_priority() {
        let mut queue = GenerationQueue::new();
        let job = create_test_job(JobPriority::Normal);
        let job_id = job.job_id.clone();

        let position = queue.add(job).unwrap();
        assert_eq!(position, 0);
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.get_position(&job_id), Some(0));
    }

    #[test]
    fn queue_add_high_priority_front() {
        let mut queue = GenerationQueue::new();

        // Add normal priority first
        let normal_job = create_test_job(JobPriority::Normal);
        let normal_id = normal_job.job_id.clone();
        queue.add(normal_job).unwrap();

        // Add high priority - should go to front
        let high_job = create_test_job(JobPriority::High);
        let high_id = high_job.job_id.clone();
        let position = queue.add(high_job).unwrap();

        assert_eq!(position, 0);
        assert_eq!(queue.get_position(&high_id), Some(0));
        assert_eq!(queue.get_position(&normal_id), Some(1));
    }

    #[test]
    fn queue_pop_next() {
        let mut queue = GenerationQueue::new();
        let job = create_test_job(JobPriority::Normal);
        let job_id = job.job_id.clone();
        queue.add(job).unwrap();

        let popped = queue.pop_next();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().job_id, job_id);
        assert!(queue.is_empty());
    }

    #[test]
    fn queue_is_full() {
        let mut queue = GenerationQueue::new();

        for _ in 0..MAX_QUEUE_SIZE {
            let job = create_test_job(JobPriority::Normal);
            queue.add(job).unwrap();
        }

        assert!(queue.is_full());
        assert_eq!(queue.len(), MAX_QUEUE_SIZE);

        // Adding another should fail
        let job = create_test_job(JobPriority::Normal);
        let result = queue.add(job);
        assert!(result.is_err());
    }

    #[test]
    fn queue_priority_ordering() {
        let mut queue = GenerationQueue::new();

        // Add 3 normal priority jobs
        let n1 = create_test_job(JobPriority::Normal);
        let n1_id = n1.job_id.clone();
        queue.add(n1).unwrap();

        let n2 = create_test_job(JobPriority::Normal);
        let n2_id = n2.job_id.clone();
        queue.add(n2).unwrap();

        let n3 = create_test_job(JobPriority::Normal);
        queue.add(n3).unwrap();

        // Add high priority - should go to front
        let h1 = create_test_job(JobPriority::High);
        let h1_id = h1.job_id.clone();
        queue.add(h1).unwrap();

        // Add another high priority - should go after first high priority
        let h2 = create_test_job(JobPriority::High);
        let h2_id = h2.job_id.clone();
        queue.add(h2).unwrap();

        // Order should be: h1, h2, n1, n2, n3
        assert_eq!(queue.get_position(&h1_id), Some(0));
        assert_eq!(queue.get_position(&h2_id), Some(1));
        assert_eq!(queue.get_position(&n1_id), Some(2));
        assert_eq!(queue.get_position(&n2_id), Some(3));
    }

    #[test]
    fn queue_positions_update_after_pop() {
        let mut queue = GenerationQueue::new();

        let j1 = create_test_job(JobPriority::Normal);
        queue.add(j1).unwrap();

        let j2 = create_test_job(JobPriority::Normal);
        let j2_id = j2.job_id.clone();
        queue.add(j2).unwrap();

        let j3 = create_test_job(JobPriority::Normal);
        let j3_id = j3.job_id.clone();
        queue.add(j3).unwrap();

        // Initial positions
        assert_eq!(queue.get_position(&j2_id), Some(1));
        assert_eq!(queue.get_position(&j3_id), Some(2));

        // Pop first job
        queue.pop_next();

        // Positions should update
        assert_eq!(queue.get_position(&j2_id), Some(0));
        assert_eq!(queue.get_position(&j3_id), Some(1));
    }

    #[test]
    fn queue_job_status_updates() {
        let mut queue = GenerationQueue::new();
        let job = create_test_job(JobPriority::Normal);

        queue.add(job).unwrap();

        // Job should be in queued status
        let job = queue.pop_next().unwrap();
        assert_eq!(job.status, JobStatus::Queued);
    }
}
