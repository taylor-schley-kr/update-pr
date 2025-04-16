use std::{cell::OnceCell, ops::Deref};

use git2::Progress;
use indicatif::{MultiProgress, ProgressBar};

pub struct FetchProgress {
    m: MultiProgress,

    object_dl_pb: OnceCell<ProgressBar>,
    object_pb: OnceCell<ProgressBar>,

    delta_pb: OnceCell<ProgressBar>,
}

impl Default for FetchProgress {
    fn default() -> Self {
        let m = MultiProgress::new();

        #[cfg(test)]
        m.set_draw_target(indicatif::ProgressDrawTarget::hidden());

        let object_dl_pb = OnceCell::new();
        let object_pb = OnceCell::new();

        let delta_pb = OnceCell::new();

        Self {
            m,

            object_dl_pb,
            object_pb,

            delta_pb,
        }
    }
}

impl FetchProgress {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&self, stats: Progress) {
        let object_dl_pb = self.object_dl_pb(stats.total_objects());
        object_dl_pb.set_position(stats.received_objects() as u64);

        let object_pb = self.object_pb(stats.total_objects());
        object_pb.set_position(stats.indexed_objects() as u64);

        let delta_pb = self.delta_pb(stats.total_deltas());
        delta_pb.set_position(stats.indexed_deltas() as u64);
    }

    pub fn object_dl_pb(&self, total: usize) -> &ProgressBar {
        self.object_dl_pb.get_or_init(|| {
            let pb = self.m.add(ProgressBar::new(total as u64));
            pb.set_style(
                indicatif::ProgressStyle::default_bar()
                    .template("{msg} {wide_bar} {eta}")
                    .unwrap()
                    .progress_chars("##-"),
            );
            pb.set_message("Downloading objects");
            self.m.add(pb)
        })
    }

    pub fn object_pb(&self, total: usize) -> &ProgressBar {
        self.object_pb.get_or_init(|| {
            let pb = self.m.add(ProgressBar::new(total as u64));

            pb.set_style(
                indicatif::ProgressStyle::default_bar()
                    .template("{msg} {wide_bar} {eta}")
                    .unwrap()
                    .progress_chars("##-"),
            );

            pb.set_message("Processing objects");
            self.m.add(pb)
        })
    }

    pub fn delta_pb(&self, total: usize) -> &ProgressBar {
        self.delta_pb.get_or_init(|| {
            let pb = self.m.add(ProgressBar::new(total as u64));

            pb.set_style(
                indicatif::ProgressStyle::default_bar()
                    .template("{msg} {wide_bar} {eta}")
                    .unwrap()
                    .progress_chars("##-"),
            );

            pb.set_message("Processing deltas");
            self.m.add(pb)
        })
    }
}

impl Deref for FetchProgress {
    type Target = MultiProgress;

    fn deref(&self) -> &Self::Target {
        &self.m
    }
}
