use crate::prelude::*;

#[derive(Reflect)]
#[reflect(dont_clone)]
#[derive(Default, Debug2)]
pub struct DownloadManager {
    #[reflect(skip)]
    pub downloads: Vec<Downloadable>,

    // a more reflect-friendly list
    pub statuses: Vec<DownloadStatus>,

    #[debug(skip)]
    #[reflect(skip)]
    pub layouts: Vec<Option<[Arc<parley::Layout<Color>>; 2]>>,
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
        self.layouts.push(None);
    }

    pub fn update(
        &mut self,
        actions: &mut ActionQueue,
        font_contexts: &mut TextLayoutContexts,
    ) {
        let mut to_remove = Vec::new();

        for (n, (dl, status, layouts)) in self
            .downloads
            .iter_mut()
            .zip(self.statuses.iter_mut())
            .zip(self.layouts.iter_mut())
            .map(|((dl, status), layouts)| (dl, status, layouts))
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

                if let Some(on_complete) = dl.on_complete.clone() {
                    actions.push(on_complete());
                }
            } else if !complete {
                let style = TextStyle::default()
                    .alignment(Alignment::CENTER)
                    .font_size(20.0);

                let mut new_progress_layout = font_contexts.simple_text(
                    &format!("{:.2}%", status.progress), 
                    &style
                );
                new_progress_layout.break_all_lines(None);

                if let Some([_, progress_layout]) = layouts {
                    *progress_layout = Arc::new(new_progress_layout);
                } else {
                    let mut title_layout = font_contexts.simple_text(
                        &status.filename, 
                        &style
                    );
                    title_layout.break_all_lines(None);

                    *layouts = Some([
                        Arc::new(title_layout),
                        Arc::new(new_progress_layout)
                    ]);
                }
            }
        }
        to_remove.reverse();

        for i in to_remove {
            self.downloads.remove(i);
            self.statuses.remove(i);
            self.layouts.remove(i);
        }
    }

    #[cfg(feature="graphics")] 
    pub fn draw(
        &self, 
        window_size: Vector2, 
        list: &mut RenderableCollection,
    ) {
        if self.statuses.is_empty() { return }

        const SIZE: Vector2 = Vector2::new(150.0, 50.0);
        for (n, i, title, progress) in self.statuses.iter()
            .zip(self.layouts.iter())
            .enumerate()
            .filter_map(|(n, (i, layouts))| 
                layouts.clone()
                    .map(|[t, p]| (n, i, t, p))
            )
        {
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

            let size = Vector2::new(SIZE.x, SIZE.y / 2.0);
            let alignment = Alignment::CENTER;
            let title_bounds = Bounds::new(pos, size);
            let title_size = Vector2::new(
                title.width(),
                title.height(),
            );


            let progress_bounds = Bounds::new(
                pos + Vector2::new(0.0, SIZE.y / 2.0),
                size
            );
            let progress_size = Vector2::new(
                progress.width(),
                progress.height(),
            );


            list.push(Transformed::new(
                Transform::default().translate(alignment.resolve(
                    &title_bounds,
                    title_size,
                    true,
                    true,
                )),
                Box::new(Text::new(title))
            ));

            list.push(Transformed::new(
                Transform::default().translate(alignment.resolve(
                    &progress_bounds,
                    progress_size,
                    true,
                    true,
                )),
                Box::new(Text::new(progress))
            ));
            // list.push(style.create_text(
            //     format!("{:.2}%", i.progress),
            //     Bounds::new(
            //         pos + Vector2::new(0.0, SIZE.y / 2.0),
            //         Vector2::new(SIZE.x, SIZE.y / 2.0)
            //     )
            // ));
        }

    }
}



#[derive(Reflect)]
#[reflect(display = "debug")]
#[derive(Clone, Debug2, Default)]
pub struct DownloadStatus {
    pub filename: String,
    pub title: String,
    pub downloading: bool,
    pub completed: bool,
    pub progress: f32,
}
