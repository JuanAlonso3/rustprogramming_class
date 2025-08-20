// Central module file that makes other modules available to the project.

// Handles website status checking
pub mod status;

// Provides input and data validation functions
pub mod validation;

// Utilities for working with time and timestamps
pub mod time_utils;

// Manages concurrent execution (running tasks in parallel)
pub mod concurrent;

// Collects and reports statistics
pub mod stats;
