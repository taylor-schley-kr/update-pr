use std::cell::OnceCell;

use indicatif::ProgressBar;

pub struct PushProgress {
    pb: OnceCell<ProgressBar>,
}

impl Default for PushProgress {
    fn default() -> Self {
        Self {
            pb: OnceCell::new(),
        }
    }
}

impl PushProgress {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pb(&self) -> &ProgressBar {
        self.pb.get_or_init(|| {
            let pb = ProgressBar::new(0);

            #[cfg(test)]
            pb.set_draw_target(indicatif::ProgressDrawTarget::hidden());

            pb.set_style(
                indicatif::ProgressStyle::default_bar()
                    .template("{wide_bar} {eta}")
                    .unwrap()
                    .progress_chars("##-"),
            );
            pb
        })
    }
}
