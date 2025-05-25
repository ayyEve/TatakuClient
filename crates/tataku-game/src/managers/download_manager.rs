use crate::prelude::*;
#[cfg(feature="graphics")] 
use tataku_engine::prelude::ui::TextStyle;

#[derive(Default, Debug)]
#[derive(Reflect)]
#[reflect(dont_clone)]
pub struct DownloadManager {
    #[reflect(skip)]
    pub downloads: Vec<Downloadable>,

    // a more reflect-friendly list
    pub statuses: Vec<DownloadStatus>,
}
impl DownloadManager {
    pub fn add_download(&mut self, mut download: Downloadable) {
        // check for duplicate download
        if self.downloads.iter().any(|i| i.filename == download.filename) { 
            return 
        }

        // TODO: queue things and only download a certain number of things at a time

        download.download_progress = Some((download.download)());
        self.statuses.push(DownloadStatus {
            filename: download.filename.clone(),
            title: download.filename.clone(),
            downloading: false,
            completed: false,
            progress: 0.0,
        });
        self.downloads.push(download);
    }

    pub fn update(
        &mut self,
        actions: &mut ActionQueue,
    ) {
        let mut to_remove = Vec::new();

        for (n, (dl, status)) in self
            .downloads
            .iter_mut()
            .zip(self.statuses.iter_mut())
            .enumerate()
        {
            status.filename = dl.filename.clone();
            status.downloading = dl.download_progress.is_some();

            let Some(p) = dl
                .download_progress
                .as_ref()
                .map(|i| i.read()) 
            else { continue };

            status.progress = p.progress();
            let complete = p.complete();

            if complete && !status.completed {
                status.completed = true;
                to_remove.push(n);

                let Some(data) = &p.data else { continue };
                std::fs::write(&dl.filename, data).unwrap();

                if let Some(on_complete) = dl.on_complete.take() {
                    actions.push(on_complete);
                }
            }
        }
        to_remove.reverse();

        for i in to_remove {
            self.downloads.remove(i);
            self.statuses.remove(i);
        }
    }

    #[cfg(feature="graphics")] 
    pub fn draw(
        &self, 
        window_size: Vector2, 
        list: &mut RenderableCollection
    ) {
        if self.statuses.is_empty() { return }

        const SIZE: Vector2 = Vector2::new(150.0, 50.0);
        for (n, i) in self.statuses.iter().enumerate() {
            let pos = window_size.x_portion() - Vector2::new(SIZE.x, -SIZE.y * n as f32);

            // progress as bg
            list.push(Rectangle::new(
                pos,
                Vector2::new(
                    SIZE.x * i.progress,
                    SIZE.y
                ),
                Color::TRANSPARENT,
            ));

            // bounds
            list.push(
                Rectangle::new(
                    pos, 
                    SIZE,
                    Color::TRANSPARENT,
                )
                .border(Border::new(Color::BLACK, 2.0))
            );

            let style = TextStyle::default()
                .alignment(Alignment::CENTER)
                .font_size(20.0);

            list.push(style.create_text(
                i.title.clone(), 
                Bounds::new(
                    pos, 
                    Vector2::new(SIZE.x, SIZE.y / 2.0)
                )
            ));

            list.push(style.create_text(
                format!("{:.2}%", i.progress), 
                Bounds::new(
                    pos + Vector2::new(0.0, SIZE.y / 2.0), 
                    Vector2::new(SIZE.x, SIZE.y / 2.0)
                )
            ));
        }

    }
}



#[derive(Reflect)]
#[derive(Clone, Debug, Default)]
#[reflect(display = "debug")]
pub struct DownloadStatus {
    pub filename: String,
    pub title: String,
    pub downloading: bool,
    pub completed: bool,
    pub progress: f32,
}
