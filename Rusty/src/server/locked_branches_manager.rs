use std::{sync::{Mutex, Arc}, io, error::Error, collections::HashSet, };
use crate::constants::ALL_BRANCHES_LOCK;
/// Manages the locking and unlocking of branches within a server protocol.
pub struct LockedBranches<'a> {
    locked_branches: &'a Arc<(Mutex<HashSet<String>>, std::sync::Condvar)>,
    current_branch_locked_branches: HashSet<String>,
}

impl<'a> LockedBranches<'a> {
    /// Creates a new instance of `LockedBranches`.
    ///
    /// # Arguments
    ///
    /// * `locked_branches` - Reference to the shared data structure containing locked branches and condition variable.
    ///
    /// # Returns
    ///
    /// A new `LockedBranches` instance.
    pub fn new(locked_branches: &'a Arc<(Mutex<HashSet<String>>, std::sync::Condvar)>) -> Self {
        LockedBranches { 
            locked_branches,
            current_branch_locked_branches: HashSet::new(),
        }
    }

    /// Locks a branch.
    ///
    /// # Arguments
    ///
    /// * `branch_to_lock` - The name of the branch to lock.
    /// * `should_extend` - Indicates whether to extend the lock if already acquired.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error if the branch could not be locked.
    pub fn lock_branch(&mut self, branch_to_lock: &str, should_extend: bool) -> Result<(), Box<dyn Error>> {
        println!("locking branch: {}", branch_to_lock);
        self._lock_branch(branch_to_lock, should_extend)?;
        self.current_branch_locked_branches.insert(branch_to_lock.to_string());

        Ok(())
    }

    /// Unlocks a branch.
    ///
    /// # Arguments
    ///
    /// * `branch_to_unlock` - The name of the branch to unlock.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error if the branch could not be unlocked.
    pub fn unlock_branch(&mut self, branch_to_unlock: &str) -> Result<(), Box<dyn Error>> {
        println!("unlocking branch: {}", branch_to_unlock);
        self._unlock_branch(branch_to_unlock)?;
        self.current_branch_locked_branches.remove(branch_to_unlock);

        Ok(())
    }

    fn _lock_branch(&mut self, branch: &str, all_branches_locked_by_me: bool) -> Result<(), Box<dyn Error>> {// Extract the Mutex and Condvar from the Arc
        // Extract the Mutex and Condvar from the Arc
        let (lock, cvar) = &**self.locked_branches;

        // Acquire the lock before checking or modifying the set of locked branches
        let mut locked_branches = match lock.lock() {
            Ok(lock) => lock,
            Err(err) => {
                // Handle the error, according to doc. Previous holder of mutex panicked
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    err.to_string(),
                )));
            }
        };

        // Wait for the branch to be available
        while locked_branches.contains(branch) || (locked_branches.contains(ALL_BRANCHES_LOCK) && !all_branches_locked_by_me) {
            println!("Branch '{}' locked going to sleep, wait for CondVar notification. Lock of HashMap released", branch);
            // Release the lock before waiting and re-acquire it after waking up
            locked_branches = match cvar.wait(locked_branches) {
                Ok(guard) => guard,
                Err(err) => {
                    // Handle the error, potentially logging or returning an error response
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        err.to_string(),
                    )));
                }
            };
            // Perform your branch-specific operations here
            println!("CondVar notification received, lock reacquired");
        }

        // Branch is not locked, so lock it
        locked_branches.insert(branch.to_string());

        // Release the lock outside the loop
        drop(locked_branches);

        println!("Branch inserted in HashMap: '{}'. Lock of HashMap released", branch);
        Ok(())
    }

    fn _unlock_branch(&mut self, branch: &str) -> Result<(), Box<dyn Error>> {
        // Extract the Mutex and Condvar from the Arc
        let (lock, cvar) = &**self.locked_branches;

        // Acquire the lock before checking or modifying the set of locked branches
        let mut locked_branches = match lock.lock() {
            Ok(lock) => lock,
            Err(err) => {
                // Handle the error, according to doc. Previous holder of mutex panicked
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    err.to_string(),
                )));
            }
        };

        // Remove the branch if it exists
        if locked_branches.remove(branch) {
            // Notify other waiting threads that the condition (branch availability) has changed
            cvar.notify_all();
        }

        // Release the lock
        drop(locked_branches);

        // Perform your branch-specific operations here
        println!("Branch removed from HashMap: '{}'. Lock of HashMap released", branch);

        Ok(())
    }
}

// Implement Drop trait for automatic unlocking
impl<'a> Drop for LockedBranches<'a> {
    fn drop(&mut self) {
        println!("dropping branches");

        for locked_branch in &self.current_branch_locked_branches.clone() {
            if self._unlock_branch(locked_branch).is_err() {
                println!("Error unlocking branch. Please restart server.")
            }
        }
    }
}

