use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ImprovementScheduler {
    pub cron_expression: Option<String>,
    pub trigger_on_session_count: Option<usize>,
    pub min_sessions_between_runs: usize,
    pub max_runs_per_day: usize,
    session_count_since_last: usize,
    runs_today: usize,
    last_run: Option<chrono::DateTime<chrono::Utc>>,
}

impl ImprovementScheduler {
    pub fn new() -> Self {
        Self {
            cron_expression: Some("0 2 * * 0".into()), // Sunday 2 AM
            trigger_on_session_count: Some(50),
            min_sessions_between_runs: 20,
            max_runs_per_day: 4,
            session_count_since_last: 0,
            runs_today: 0,
            last_run: None,
        }
    }

    pub fn with_cron(mut self, cron: &str) -> Self {
        self.cron_expression = Some(cron.into());
        self
    }

    pub fn with_trigger(mut self, count: usize) -> Self {
        self.trigger_on_session_count = Some(count);
        self
    }

    pub fn with_min_sessions(mut self, count: usize) -> Self {
        self.min_sessions_between_runs = count;
        self
    }

    pub fn runs_today(&self) -> usize {
        self.runs_today
    }

    pub fn notify_session_completed(&mut self) {
        self.session_count_since_last += 1;
    }

    /// Check if improvement should run now
    pub fn should_run(&mut self) -> bool {
        // Daily limit
        let today = chrono::Utc::now().date_naive();
        if let Some(last) = self.last_run {
            if last.date_naive() != today {
                self.runs_today = 0;
            }
        }
        if self.runs_today >= self.max_runs_per_day {
            return false;
        }

        // Trigger by session count
        if let Some(threshold) = self.trigger_on_session_count {
            if self.session_count_since_last >= threshold {
                return true;
            }
        }

        // Cron check (simplified — just checks if enough sessions)
        if self.session_count_since_last >= self.min_sessions_between_runs {
            return true;
        }

        false
    }

    pub fn mark_ran(&mut self) {
        self.session_count_since_last = 0;
        self.runs_today += 1;
        self.last_run = Some(chrono::Utc::now());
    }

    /// Time until next scheduled run (for cron-based scheduling)
    pub fn next_run_in(&self) -> Option<Duration> {
        if let Some(ref _cron) = self.cron_expression {
            // Simplified: suggest in ~1 hour if enough sessions
            if self.session_count_since_last >= self.min_sessions_between_runs {
                return Some(Duration::from_secs(0));
            }
            let remaining = self.min_sessions_between_runs - self.session_count_since_last;
            Some(Duration::from_secs(remaining as u64 * 60)) // ~1 min per session estimate
        } else {
            None
        }
    }
}

impl Default for ImprovementScheduler {
    fn default() -> Self {
        Self::new()
    }
}
