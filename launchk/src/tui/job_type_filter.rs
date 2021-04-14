use std::fmt;
use std::fmt::Formatter;

bitflags! {
    #[derive(Default)]
    /// Bitmask for filtering on the job type, which is a mix
    /// of scope (where it's located), and kind (agent v. daemon)
    pub struct JobTypeFilter: u32 {
        const SYSTEM = (1 << 1);
        const GLOBAL = (1 << 2);
        const USER   = (1 << 3);
        const AGENT  = (1 << 4);
        const DAEMON = (1 << 5);
    }
}

/// Represent the bitmask as a string for easy TUI check for styling
/// hotkey status
impl fmt::Display for JobTypeFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut display = format!("");

        if (*self & JobTypeFilter::SYSTEM) == JobTypeFilter::SYSTEM {
            display.push('s');
        }

        if (*self & JobTypeFilter::GLOBAL) == JobTypeFilter::GLOBAL {
            display.push('g');
        }

        if (*self & JobTypeFilter::USER) == JobTypeFilter::USER {
            display.push('u');
        }

        if (*self & JobTypeFilter::AGENT) == JobTypeFilter::AGENT {
            display.push('a');
        }

        if (*self & JobTypeFilter::DAEMON) == JobTypeFilter::DAEMON {
            display.push('d');
        }

        write!(f, "{}", display)
    }
}