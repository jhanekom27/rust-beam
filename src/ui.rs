use indicatif::{ProgressBar, ProgressStyle};

use crate::transmission::UpdateProgress;

pub struct ProgressBarTracker {
    pub progress_bar: ProgressBar,
}

impl ProgressBarTracker {
    pub fn new(total_bytes: u64) -> Self {
        let progress_bar = ProgressBar::new(total_bytes);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap(),
        );
        ProgressBarTracker { progress_bar }
    }

    pub fn done(self) {
        self.progress_bar.finish_with_message("Done");
    }
}

impl UpdateProgress for ProgressBarTracker {
    fn update_progress(&mut self, bytes_read: u64) {
        self.progress_bar.set_position(bytes_read);
    }
}
