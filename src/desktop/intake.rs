use std::path::PathBuf;

use rfd::AsyncFileDialog;
use slint::DataTransfer;

pub(super) enum Picker {
    Videos,
    Destination,
}

pub(super) enum Selection {
    Videos(Vec<PathBuf>),
    Destination(Option<PathBuf>),
}

pub(super) async fn select(picker: Picker) -> Selection {
    match picker {
        Picker::Videos => Selection::Videos(
            AsyncFileDialog::new()
                .set_title("Selecionar vídeos")
                .add_filter(
                    "Vídeos",
                    &[
                        "mp4", "mkv", "webm", "mov", "avi", "m4v", "mpg", "mpeg", "ts", "mxf",
                    ],
                )
                .add_filter("Todos os arquivos", &["*"])
                .pick_files()
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|file| file.path().to_path_buf())
                .collect(),
        ),
        Picker::Destination => Selection::Destination(
            AsyncFileDialog::new()
                .set_title("Escolher pasta de destino")
                .pick_folder()
                .await
                .map(|folder| folder.path().to_path_buf()),
        ),
    }
}

pub(super) fn dropped_paths(data: DataTransfer) -> Vec<PathBuf> {
    data.file_paths()
        .map(|paths| paths.map(ToOwned::to_owned).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::SharedString;

    #[test]
    fn accepts_multiple_native_paths_in_order() {
        let expected = [
            PathBuf::from("/tmp/um vídeo.mp4"),
            PathBuf::from("/tmp/dois.mkv"),
        ];
        let mut transfer = DataTransfer::default();
        transfer.set_file_paths(&expected);
        assert_eq!(dropped_paths(transfer), expected);
    }

    #[test]
    fn rejects_plain_text_as_file_input() {
        let transfer = DataTransfer::from(SharedString::from("file:///tmp/video.mp4"));
        assert!(dropped_paths(transfer).is_empty());
    }
}
